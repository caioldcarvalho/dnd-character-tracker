// Central app state (Svelte 5 runes). Holds the editable CharacterSheet and the
// engine-derived ComputedCharacter; recomputes via IPC on every sheet change.
//
// The store is the single contract the build UI mutates through: every editor
// calls one of the mutation helpers, which edit the sheet, recompute, and mark
// the document dirty (with optional debounced autosave).

import * as ipc from './ipc';
import type { Catalog } from '$bindings';

type Sheet = any;
type Computed = any;
type Breakdown = any;
type PendingChoice = any;

/** True when running inside the Tauri shell (so `invoke` exists). */
export function inTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

class AppState {
  sheet = $state<Sheet | null>(null);
  computed = $state<Computed | null>(null);
  /** Loaded content summaries for the build pickers (fetched once). */
  catalog = $state<Catalog | null>(null);
  /** Path of the loaded character file, if any. */
  path = $state<string | null>(null);
  /** Section shown in the left rail. */
  section = $state<'sheet' | 'build' | 'spells' | 'gear' | 'notes'>('sheet');
  /** The stat whose breakdown is pinned in the inspector. */
  inspecting = $state<{ stat: any; breakdown: Breakdown } | null>(null);
  /** Whether the last compute hit an error. */
  error = $state<string | null>(null);
  busy = $state(false);
  /** Unsaved changes since the last save. */
  dirty = $state(false);
  saving = $state(false);

  #saveTimer: ReturnType<typeof setTimeout> | null = null;

  // ---- derived helpers ----

  get pendingChoices(): PendingChoice[] {
    return this.computed?.pending_choices ?? [];
  }

  // ---- lifecycle ----

  /** Fetch the content catalog once (for build pickers). */
  async ensureCatalog() {
    if (this.catalog) return;
    try {
      this.catalog = await ipc.catalog();
    } catch (e) {
      this.error = String(e);
    }
  }

  /** Load a sheet and compute it. */
  async setSheet(sheet: Sheet, path: string | null = null) {
    this.sheet = sheet;
    this.path = path;
    this.dirty = false;
    await this.recompute();
  }

  /** Start a brand-new character and switch to Build mode. */
  async newCharacter(name = 'New Character') {
    try {
      const sheet = inTauri() ? await ipc.newSheet(name) : blankSheet(name);
      await this.setSheet(sheet, null);
      this.section = 'build';
      this.dirty = true;
    } catch (e) {
      this.error = String(e);
    }
  }

  /** Re-run the engine over the current sheet. */
  async recompute() {
    if (!this.sheet) return;
    this.busy = true;
    this.error = null;
    try {
      this.computed = await ipc.compute(this.sheet);
      if (this.inspecting) {
        this.inspecting = {
          stat: this.inspecting.stat,
          breakdown: await ipc.explain(this.sheet, this.inspecting.stat)
        };
      }
    } catch (e) {
      this.error = String(e);
    } finally {
      this.busy = false;
    }
  }

  /** Apply a sheet edit: recompute, mark dirty, schedule autosave. */
  async #edit(mutate: (s: Sheet) => void) {
    if (!this.sheet) return;
    mutate(this.sheet);
    this.dirty = true;
    await this.recompute();
    this.#scheduleAutosave();
  }

  // ---- identity mutations ----

  setName(name: string) {
    return this.#edit((s) => (s.meta.name = name));
  }
  setSpecies(id: string | null) {
    return this.#edit((s) => (s.species = id));
  }
  setBackground(id: string | null) {
    return this.#edit((s) => (s.background = id));
  }
  setAbility(ability: string, score: number) {
    return this.#edit((s) => (s.abilities[ability] = score));
  }
  setAbilities(scores: Record<string, number>) {
    return this.#edit((s) => (s.abilities = { ...scores }));
  }

  // ---- class mutations ----

  addClass(classId: string) {
    return this.#edit((s) => {
      s.classes.push({ class: classId, level: 1, subclass: null });
    });
  }
  removeClass(index: number) {
    return this.#edit((s) => s.classes.splice(index, 1));
  }
  setClassLevel(index: number, level: number) {
    return this.#edit((s) => {
      const c = s.classes[index];
      if (c) c.level = Math.max(1, Math.min(20, level));
    });
  }
  setSubclass(index: number, subclassId: string | null) {
    return this.#edit((s) => {
      const c = s.classes[index];
      if (c) c.subclass = subclassId;
    });
  }

  // ---- choices ----

  /**
   * Resolve a pending choice. Subclass choices set the matching ClassEntry's
   * subclass; everything else writes a RecordedChoice {key, picks}.
   */
  resolveChoice(choice: PendingChoice, picks: string[]) {
    return this.#edit((s) => {
      if (choice.options?.kind === 'subclass') {
        // key is "{class}-subclass"; set that class entry's subclass.
        const classId = choice.key.replace(/-subclass$/, '');
        const entry = s.classes.find((c: any) => c.class === classId);
        if (entry) entry.subclass = picks[0] ?? null;
        return;
      }
      const existing = s.choices.find((c: any) => c.key === choice.key);
      if (existing) existing.picks = picks;
      else s.choices.push({ key: choice.key, picks });
    });
  }

  /** Drop a recorded choice (re-opens it as pending). */
  clearChoice(key: string) {
    return this.#edit((s) => {
      s.choices = s.choices.filter((c: any) => c.key !== key);
    });
  }

  /** The picks already recorded for a choice key (for pre-selecting controls). */
  picksFor(choice: PendingChoice): string[] {
    if (!this.sheet) return [];
    if (choice.options?.kind === 'subclass') {
      const classId = choice.key.replace(/-subclass$/, '');
      const entry = this.sheet.classes.find((c: any) => c.class === classId);
      return entry?.subclass ? [entry.subclass] : [];
    }
    return this.sheet.choices.find((c: any) => c.key === choice.key)?.picks ?? [];
  }

  // ---- persistence ----

  #scheduleAutosave() {
    if (!inTauri() || !this.path) return; // only autosave already-saved files
    if (this.#saveTimer) clearTimeout(this.#saveTimer);
    this.#saveTimer = setTimeout(() => this.save(), 900);
  }

  /** Persist to disk. Creates the file on first save (path returned by backend). */
  async save() {
    if (!this.sheet || !inTauri()) return;
    this.saving = true;
    try {
      const saved = await ipc.saveCharacter(this.path, this.sheet);
      if (saved) this.path = saved;
      this.dirty = false;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.saving = false;
    }
  }

  // ---- inspector ----

  async inspect(stat: any) {
    if (!this.sheet) return;
    try {
      const breakdown = await ipc.explain(this.sheet, stat);
      this.inspecting = { stat, breakdown };
    } catch (e) {
      this.error = String(e);
    }
  }

  closeInspector() {
    this.inspecting = null;
  }
}

/** A blank sheet for browser-preview (when the Tauri backend is absent). */
function blankSheet(name: string): Sheet {
  const abilities: Record<string, number> = { str: 10, dex: 10, con: 10, int: 10, wis: 10, cha: 10 };
  return {
    meta: { name, player: '', id: '' },
    abilities,
    species: null,
    background: null,
    classes: [],
    feats: [],
    choices: [],
    hp: { current: 0, temp: 0, rolled: [] },
    resources: {},
    hit_dice_spent: {},
    conditions: [],
    exhaustion: 0,
    active_effects: [],
    equipment: { armor: null, shield: false },
    weapons: [],
    concentration: null,
    death_saves: { successes: 0, failures: 0 },
    inspiration: false
  };
}

export const app = new AppState();

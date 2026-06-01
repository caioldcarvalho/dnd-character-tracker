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
  inspecting = $state<{
    stat: any;
    breakdown: Breakdown;
    weaponRef: { index: number; mode: 'attack' | 'damage' } | null;
  } | null>(null);
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

  /** Close the current character and return to the library/open menu. */
  async closeCharacter(opts: { saveFirst?: boolean } = {}) {
    if (opts.saveFirst && this.dirty && inTauri()) {
      await this.save();
    }
    if (this.#saveTimer) {
      clearTimeout(this.#saveTimer);
      this.#saveTimer = null;
    }
    this.sheet = null;
    this.computed = null;
    this.path = null;
    this.dirty = false;
    this.inspecting = null;
    this.error = null;
    this.section = 'sheet';
  }

  /** Re-run the engine over the current sheet. */
  async recompute() {
    if (!this.sheet) return;
    this.busy = true;
    this.error = null;
    try {
      this.computed = await ipc.compute(this.sheet);
      // While building, current HP tracks max (you're not playing yet, so the
      // sheet should read full). Max HP never depends on current HP, so this
      // assignment can't cause a recompute loop. Play mode preserves current HP.
      if (this.section === 'build' && this.sheet) {
        this.sheet.hp.current = this.computed?.max_hp?.total ?? 0;
      }
      // Refresh an open inspector against the new computation.
      if (this.inspecting?.weaponRef) {
        const { index, mode } = this.inspecting.weaponRef;
        const w = this.computed?.weapons?.[index];
        if (w) {
          this.inspecting = {
            stat: null,
            weaponRef: this.inspecting.weaponRef,
            breakdown: mode === 'attack' ? w.attack_breakdown : w.damage_breakdown
          };
        } else {
          this.inspecting = null; // weapon removed
        }
      } else if (this.inspecting?.stat) {
        this.inspecting = {
          stat: this.inspecting.stat,
          weaponRef: null,
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

  // ---- gear: weapons ----

  addWeapon(w?: Partial<any>) {
    return this.#edit((s) => {
      s.weapons.push({
        name: w?.name ?? 'New Weapon',
        kind: w?.kind ?? 'melee',
        damage: w?.damage ?? { count: 1, sides: 6 },
        damage_type: w?.damage_type ?? 'slashing',
        finesse: w?.finesse ?? false,
        two_handed: w?.two_handed ?? false,
        magic_bonus: w?.magic_bonus ?? 0,
        proficient: w?.proficient ?? true,
        mastery: w?.mastery ?? null
      });
    });
  }
  updateWeapon(index: number, patch: Partial<any>) {
    return this.#edit((s) => {
      const w = s.weapons[index];
      if (w) Object.assign(w, patch);
    });
  }
  removeWeapon(index: number) {
    return this.#edit((s) => s.weapons.splice(index, 1));
  }

  // ---- gear: armor & shield ----

  /** Set worn armor (null = unarmored). */
  setArmor(armor: any | null) {
    return this.#edit((s) => {
      s.equipment.armor = armor;
    });
  }
  toggleShield() {
    return this.#edit((s) => {
      s.equipment.shield = !s.equipment.shield;
    });
  }

  // ---- notes ----

  setNotes(text: string) {
    return this.#edit((s) => {
      s.notes = text;
    });
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

  // ---- play: HP ----

  get maxHp(): number {
    return this.computed?.max_hp?.total ?? 0;
  }

  /** Apply damage: depletes temp HP first, then current HP (floored at 0). */
  applyDamage(amount: number) {
    const n = Math.max(0, Math.floor(amount));
    return this.#edit((s) => {
      let dmg = n;
      const absorbed = Math.min(s.hp.temp, dmg);
      s.hp.temp -= absorbed;
      dmg -= absorbed;
      s.hp.current = Math.max(0, s.hp.current - dmg);
    });
  }

  /** Heal: raises current HP, clamped to max. */
  heal(amount: number) {
    const n = Math.max(0, Math.floor(amount));
    return this.#edit((s) => {
      s.hp.current = Math.min(this.maxHp, s.hp.current + n);
    });
  }

  /** Set temp HP (2024: temp HP doesn't stack — the higher value wins). */
  setTempHp(amount: number) {
    const n = Math.max(0, Math.floor(amount));
    return this.#edit((s) => {
      s.hp.temp = Math.max(s.hp.temp, n);
    });
  }

  /** Directly set current HP (manual override), clamped 0..max. */
  setCurrentHp(value: number) {
    return this.#edit((s) => {
      s.hp.current = Math.max(0, Math.min(this.maxHp, Math.floor(value)));
    });
  }

  // ---- play: resources ----

  /** Spend `n` from a resource pool (clamped at 0). */
  spendResource(id: string, max: number, n = 1) {
    return this.#edit((s) => {
      const cur = s.resources[id] ?? max;
      s.resources[id] = Math.max(0, cur - n);
    });
  }
  /** Restore `n` to a resource pool (clamped at max). */
  restoreResource(id: string, max: number, n = 1) {
    return this.#edit((s) => {
      const cur = s.resources[id] ?? max;
      const next = Math.min(max, cur + n);
      if (next >= max) delete s.resources[id]; // full → drop to keep sheet minimal
      else s.resources[id] = next;
    });
  }
  /** Set a resource's current value directly (clamped 0..max). */
  setResource(id: string, max: number, value: number) {
    return this.#edit((s) => {
      const v = Math.max(0, Math.min(max, Math.floor(value)));
      if (v >= max) delete s.resources[id];
      else s.resources[id] = v;
    });
  }

  // ---- play: spell slots ----

  expendSlot(level: number) {
    return this.#edit((s) => {
      s.slots_expended[level] = (s.slots_expended[level] ?? 0) + 1;
    });
  }
  restoreSlot(level: number) {
    return this.#edit((s) => {
      const cur = s.slots_expended[level] ?? 0;
      if (cur <= 1) delete s.slots_expended[level];
      else s.slots_expended[level] = cur - 1;
    });
  }

  // ---- play: hit dice ----

  spendHitDie(sides: number, total: number) {
    return this.#edit((s) => {
      const spent = s.hit_dice_spent[sides] ?? 0;
      if (spent < total) s.hit_dice_spent[sides] = spent + 1;
    });
  }

  // ---- play: rests ----

  async rest(kind: 'short' | 'long') {
    if (!this.sheet) return;
    if (inTauri()) {
      // Authoritative recharge in Rust.
      try {
        const rested = await ipc.rest(this.sheet, kind);
        this.sheet = rested;
        this.dirty = true;
        await this.recompute();
        this.#scheduleAutosave();
      } catch (e) {
        this.error = String(e);
      }
    } else {
      // Browser-preview fallback: clear the relevant runtime state locally.
      await this.#edit((s) => {
        s.resources = {};
        if (kind === 'long') {
          s.slots_expended = {};
          s.hp.current = this.maxHp;
          s.hp.temp = 0;
          s.exhaustion = Math.max(0, (s.exhaustion ?? 0) - 1);
          s.concentration = null;
        }
      });
    }
  }

  // ---- play: conditions / effects / status ----

  toggleCondition(name: string) {
    return this.#edit((s) => {
      const i = s.conditions.indexOf(name);
      if (i >= 0) s.conditions.splice(i, 1);
      else s.conditions.push(name);
    });
  }
  setExhaustion(level: number) {
    return this.#edit((s) => (s.exhaustion = Math.max(0, Math.min(6, Math.floor(level)))));
  }
  toggleEffect(id: string) {
    return this.#edit((s) => {
      const i = s.active_effects.indexOf(id);
      if (i >= 0) s.active_effects.splice(i, 1);
      else s.active_effects.push(id);
    });
  }
  setConcentration(spell: string | null) {
    return this.#edit((s) => (s.concentration = spell));
  }
  toggleInspiration() {
    return this.#edit((s) => (s.inspiration = !s.inspiration));
  }
  /** Record a death save tick (kind: 'success' | 'failure'), or reset. */
  deathSave(kind: 'success' | 'failure' | 'reset') {
    return this.#edit((s) => {
      if (kind === 'reset') {
        s.death_saves = { successes: 0, failures: 0 };
      } else if (kind === 'success') {
        s.death_saves.successes = Math.min(3, (s.death_saves.successes ?? 0) + 1);
      } else {
        s.death_saves.failures = Math.min(3, (s.death_saves.failures ?? 0) + 1);
      }
    });
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

  /** Inspect a graph StatId (fetched via explain, refreshed on recompute). */
  async inspect(stat: any) {
    if (!this.sheet) return;
    try {
      const breakdown = await ipc.explain(this.sheet, stat);
      this.inspecting = { stat, breakdown, weaponRef: null };
    } catch (e) {
      this.error = String(e);
    }
  }

  /**
   * Inspect a per-weapon attack/damage breakdown. These are computed in the
   * view (not the DAG), so we pull them from `computed.weapons` and re-pull on
   * recompute via the stored ref.
   */
  inspectWeapon(index: number, mode: 'attack' | 'damage') {
    const w = this.computed?.weapons?.[index];
    if (!w) return;
    const breakdown = mode === 'attack' ? w.attack_breakdown : w.damage_breakdown;
    this.inspecting = { stat: null, breakdown, weaponRef: { index, mode } };
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
    inspiration: false,
    notes: ''
  };
}

export const app = new AppState();

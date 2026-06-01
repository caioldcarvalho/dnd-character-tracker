// Central app state (Svelte 5 runes). Holds the editable CharacterSheet and the
// engine-derived ComputedCharacter; recomputes via IPC on every sheet change.
//
// Types are intentionally loose (`any`) where the ts-rs bindings aren't imported
// yet; the IPC layer (ipc.ts) is the typed boundary. This keeps the UI buildable
// while bindings are generated, and the shapes match the engine's serde output.

import * as ipc from './ipc';

type Sheet = any;
type Computed = any;
type Breakdown = any;

/** True when running inside the Tauri shell (so `invoke` exists). */
export function inTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

class AppState {
  sheet = $state<Sheet | null>(null);
  computed = $state<Computed | null>(null);
  /** Path of the loaded character file, if any. */
  path = $state<string | null>(null);
  /** Section shown in the left rail. */
  section = $state<'sheet' | 'build' | 'spells' | 'gear' | 'notes'>('sheet');
  /** The stat whose breakdown is pinned in the inspector. */
  inspecting = $state<{ stat: any; breakdown: Breakdown } | null>(null);
  /** Whether the last compute hit an error. */
  error = $state<string | null>(null);
  busy = $state(false);

  /** Load a sheet and compute it. */
  async setSheet(sheet: Sheet, path: string | null = null) {
    this.sheet = sheet;
    this.path = path;
    await this.recompute();
  }

  /** Re-run the engine over the current sheet. */
  async recompute() {
    if (!this.sheet) return;
    this.busy = true;
    this.error = null;
    try {
      this.computed = await ipc.compute(this.sheet);
      // Refresh an open inspector against the new computation.
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

  /** Open (or replace) the inspector with a stat's breakdown. */
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

export const app = new AppState();

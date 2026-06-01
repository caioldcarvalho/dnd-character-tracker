// Hand-written wrappers over the Tauri command surface. Types come from the
// ts-rs-generated bindings; this file is the only place `invoke` is called.
//
// Until the Rust commands are wired (M0), these are typed against the bindings
// and will throw at runtime if called before the backend exists — the UI guards
// for that during early milestones.

import { invoke } from '@tauri-apps/api/core';
import type {
  Catalog,
  CharacterSheet,
  ComputedCharacter,
  StatBreakdown,
  StatId
} from '$bindings';

/** Summaries of all content (classes/species/etc.) for the build pickers. */
export function catalog(): Promise<Catalog> {
  return invoke('catalog');
}

/** Derive the full computed sheet. Called on every edit. */
export function compute(sheet: CharacterSheet): Promise<ComputedCharacter> {
  return invoke('compute', { sheet });
}

/** The contribution breakdown for a single stat (the inspector panel). */
export function explain(sheet: CharacterSheet, stat: StatId): Promise<StatBreakdown> {
  return invoke('explain', { sheet, stat });
}

/** A blank character. */
export function newSheet(name: string): Promise<CharacterSheet> {
  return invoke('new_sheet', { name });
}

export function listCharacters(): Promise<{ name: string; path: string }[]> {
  return invoke('list_characters');
}

export function loadCharacter(path: string): Promise<CharacterSheet> {
  return invoke('load_character', { path });
}

/** Persist a character. `path` null creates a new file; returns the saved path. */
export function saveCharacter(path: string | null, sheet: CharacterSheet): Promise<string> {
  return invoke('save_character', { path, sheet });
}

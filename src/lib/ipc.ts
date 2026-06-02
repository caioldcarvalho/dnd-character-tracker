// The one bridge to the rules engine. Inside the Tauri shell these call Rust via
// `invoke`; in a plain browser they call the *same* engine compiled to WebAssembly
// (`crates/wasm`), so the app is fully functional as a pure web build. Nothing else
// in the app knows or cares which backend is live — the switch lives here.
//
// Types come from the ts-rs-generated bindings, shared by both backends (the WASM
// functions are JSON-string in / JSON-string out, serializing the same serde types
// the Tauri commands return).

import { invoke } from '@tauri-apps/api/core';
import type {
  Catalog,
  CharacterSheet,
  ComputedCharacter,
  StatBreakdown,
  StatId
} from '$bindings';

/** True when running inside the Tauri shell (so `invoke` exists). */
function inTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

// ---- WASM backend (browser mode) ----

type WasmModule = typeof import('$lib/wasm/rpgman_wasm');
let wasmPromise: Promise<WasmModule> | null = null;

/** Lazily fetch + instantiate the WASM engine. Browser mode only; loaded once. */
function wasm(): Promise<WasmModule> {
  if (!wasmPromise) {
    wasmPromise = import('$lib/wasm/rpgman_wasm').then(async (m) => {
      await m.default(); // init(): fetch + instantiate the .wasm, run the start hook
      return m;
    });
  }
  return wasmPromise;
}

// ---- engine commands ----

/** Summaries of all content (classes/species/etc.) for the build pickers. */
export async function catalog(): Promise<Catalog> {
  if (inTauri()) return invoke('catalog');
  return JSON.parse((await wasm()).catalog());
}

/** Derive the full computed sheet. Called on every edit. */
export async function compute(sheet: CharacterSheet): Promise<ComputedCharacter> {
  if (inTauri()) return invoke('compute', { sheet });
  return JSON.parse((await wasm()).compute(JSON.stringify(sheet)));
}

/** The contribution breakdown for a single stat (the inspector panel). */
export async function explain(sheet: CharacterSheet, stat: StatId): Promise<StatBreakdown> {
  if (inTauri()) return invoke('explain', { sheet, stat });
  return JSON.parse((await wasm()).explain(JSON.stringify(sheet), JSON.stringify(stat)));
}

/** Apply a short or long rest; returns the updated sheet. */
export async function rest(sheet: CharacterSheet, kind: 'short' | 'long'): Promise<CharacterSheet> {
  if (inTauri()) return invoke('rest', { sheet, kind });
  return JSON.parse((await wasm()).rest(JSON.stringify(sheet), kind));
}

/** A blank character. */
export async function newSheet(name: string): Promise<CharacterSheet> {
  if (inTauri()) return invoke('new_sheet', { name });
  return JSON.parse((await wasm()).new_sheet(name));
}

// ---- persistence ----
// The desktop app saves character files to disk (Tauri commands). The browser has
// no filesystem, so characters live in localStorage. The store gates autosave/save
// on `inTauri()`, but we implement the browser path too so an embedded/hosted web
// build can persist (and so a future GitHub Pages deploy just works).

const LS_PREFIX = 'rpgman:char:';

export async function listCharacters(): Promise<{ name: string; path: string }[]> {
  if (inTauri()) return invoke('list_characters');
  const out: { name: string; path: string }[] = [];
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (!key || !key.startsWith(LS_PREFIX)) continue;
    try {
      const sheet = JSON.parse(localStorage.getItem(key) ?? '');
      out.push({ name: sheet?.meta?.name || key.slice(LS_PREFIX.length), path: key });
    } catch {
      /* skip a corrupt entry */
    }
  }
  out.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));
  return out;
}

export async function loadCharacter(path: string): Promise<CharacterSheet> {
  if (inTauri()) return invoke('load_character', { path });
  const text = localStorage.getItem(path);
  if (text == null) throw new Error(`character not found: ${path}`);
  return JSON.parse(text);
}

/** Persist a character. `path` null creates a new entry; returns its key/path. */
export async function saveCharacter(path: string | null, sheet: CharacterSheet): Promise<string> {
  if (inTauri()) return invoke('save_character', { path, sheet });
  const key = path ?? `${LS_PREFIX}${slug((sheet as { meta?: { name?: string } })?.meta?.name)}`;
  localStorage.setItem(key, JSON.stringify(sheet));
  return key;
}

function slug(name: string | undefined): string {
  const s = (name ?? '').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '');
  return s || 'character';
}

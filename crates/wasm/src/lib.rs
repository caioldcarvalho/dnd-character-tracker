//! WebAssembly bindings for the rpgman engine — lets the SvelteKit frontend run
//! the *same* rules engine in a plain browser, with no Tauri shell, so the app
//! works as a pure web build. This mirrors the Tauri command surface in
//! `src-tauri/src/lib.rs` one-for-one.
//!
//! Every function is JSON-string in / JSON-string out, which matches exactly what
//! the Tauri `invoke` path returns (both serialize the same serde types). The TS
//! side (`src/lib/ipc.ts`) chooses WASM vs `invoke` from the runtime environment,
//! so nothing else in the app changes.

use rpgman_engine::{
    compute as engine_compute, explain as engine_explain, rest as engine_rest, Catalog,
    CharacterSheet, ContentDb, RestKind, StatId,
};
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

/// The embedded content database (baked into the `.wasm` by the engine's
/// `build.rs`), loaded once. The browser has no filesystem, so the embedded copy
/// is all there is — which is the very same content the packaged desktop app
/// ships with, so web and desktop compute identically.
fn content() -> &'static ContentDb {
    static DB: OnceLock<ContentDb> = OnceLock::new();
    DB.get_or_init(|| ContentDb::embedded().unwrap_or_default())
}

#[wasm_bindgen(start)]
pub fn start() {
    // Surface Rust panics as readable console errors instead of bare "unreachable".
    console_error_panic_hook::set_once();
}

/// Content summaries for the build pickers. Mirrors the `catalog` command.
#[wasm_bindgen]
pub fn catalog() -> Result<String, JsError> {
    Ok(serde_json::to_string(&Catalog::from_content(content()))?)
}

/// Derive the full computed sheet. Mirrors the `compute` command.
#[wasm_bindgen]
pub fn compute(sheet_json: &str) -> Result<String, JsError> {
    let sheet: CharacterSheet = serde_json::from_str(sheet_json)?;
    Ok(serde_json::to_string(&engine_compute(&sheet, content()))?)
}

/// The contribution breakdown for one stat. Mirrors the `explain` command.
#[wasm_bindgen]
pub fn explain(sheet_json: &str, stat_json: &str) -> Result<String, JsError> {
    let sheet: CharacterSheet = serde_json::from_str(sheet_json)?;
    let stat: StatId = serde_json::from_str(stat_json)?;
    Ok(serde_json::to_string(&engine_explain(&sheet, content(), &stat))?)
}

/// Apply a short or long rest, returning the updated sheet. Mirrors `rest`.
#[wasm_bindgen]
pub fn rest(sheet_json: &str, kind: &str) -> Result<String, JsError> {
    let sheet: CharacterSheet = serde_json::from_str(sheet_json)?;
    let kind = RestKind::parse(kind)
        .ok_or_else(|| JsError::new(&format!("unknown rest kind: {kind}")))?;
    Ok(serde_json::to_string(&engine_rest(&sheet, content(), kind))?)
}

/// A blank character. Mirrors the `new_sheet` command.
#[wasm_bindgen]
pub fn new_sheet(name: &str) -> Result<String, JsError> {
    Ok(serde_json::to_string(&CharacterSheet::new(name))?)
}

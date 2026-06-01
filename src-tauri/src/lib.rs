//! The Tauri shell. Thin command layer over `rpgman-engine`: the engine does all
//! the rules work; this crate handles content loading, character file I/O, and
//! exposing engine functions to the SvelteKit frontend.

mod content_embed;

use rpgman_engine::{
    compute as engine_compute, explain as engine_explain, Catalog, CharacterSheet,
    ComputedCharacter, ContentDb, StatBreakdown, StatId,
};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The content database, loaded once. Embedded at compile time so the packaged
/// app needs no external files; can be overridden by a `content/` dir next to the
/// executable for user homebrew (checked first).
fn content() -> &'static ContentDb {
    static DB: OnceLock<ContentDb> = OnceLock::new();
    DB.get_or_init(content_embed::load)
}

#[derive(serde::Serialize)]
struct CharacterEntry {
    name: String,
    path: String,
}

// ---- commands ----

#[tauri::command]
fn catalog() -> Catalog {
    Catalog::from_content(content())
}

#[tauri::command]
fn compute(sheet: CharacterSheet) -> ComputedCharacter {
    engine_compute(&sheet, content())
}

#[tauri::command]
fn explain(sheet: CharacterSheet, stat: StatId) -> StatBreakdown {
    engine_explain(&sheet, content(), &stat)
}

#[tauri::command]
fn new_sheet(name: String) -> CharacterSheet {
    CharacterSheet::new(name)
}

/// Directory holding saved characters, created on first use.
fn characters_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("characters");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

#[tauri::command]
fn list_characters(app: tauri::AppHandle) -> Result<Vec<CharacterEntry>, String> {
    let dir = characters_dir(&app)?;
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        // Prefer the character's own name; fall back to the file stem.
        let name = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<CharacterSheet>(&t).ok())
            .map(|s| s.meta.name)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| path.file_stem().unwrap().to_string_lossy().into_owned());
        out.push(CharacterEntry {
            name,
            path: path.to_string_lossy().into_owned(),
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

#[tauri::command]
fn load_character(path: String) -> Result<CharacterSheet, String> {
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_character(
    app: tauri::AppHandle,
    path: Option<String>,
    sheet: CharacterSheet,
) -> Result<String, String> {
    let target = match path {
        Some(p) => PathBuf::from(p),
        None => {
            // New file: slugify the character name into the characters dir.
            let dir = characters_dir(&app)?;
            let slug: String = sheet
                .meta
                .name
                .chars()
                .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
                .collect();
            let slug = if slug.trim_matches('-').is_empty() { "character".into() } else { slug };
            dir.join(format!("{slug}.json"))
        }
    };
    let json = serde_json::to_string_pretty(&sheet).map_err(|e| e.to_string())?;
    std::fs::write(&target, json).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            catalog,
            compute,
            explain,
            new_sheet,
            list_characters,
            load_character,
            save_character,
        ])
        .run(tauri::generate_context!())
        .expect("error while running rpgman");
}

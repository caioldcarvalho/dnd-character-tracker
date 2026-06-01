//! Content loading for the packaged app. The rules data under `content/` is
//! embedded into the binary at compile time (via `include_dir`-style inlining
//! done by a build step) so the app ships self-contained. For development and
//! homebrew, a `content/` directory found next to the executable or at the repo
//! root takes precedence.

use rpgman_engine::ContentDb;
use std::path::{Path, PathBuf};

/// Locate a usable `content/` directory, or fall back to the embedded copy.
pub fn load() -> ContentDb {
    if let Some(dir) = find_content_dir() {
        if let Ok(db) = ContentDb::load_dir(&dir) {
            return db;
        }
    }
    embedded()
}

/// Search likely locations for a content directory (dev convenience + homebrew).
fn find_content_dir() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // Next to the executable (packaged homebrew override).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("content"));
        }
    }
    // The repo root during `tauri dev` (CARGO_MANIFEST_DIR = src-tauri).
    candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../content"));
    // Current working directory.
    candidates.push(PathBuf::from("content"));

    candidates.into_iter().find(|p| p.join("classes").is_dir())
}

/// The compile-time-embedded content, built by build.rs into `content_gen.rs`.
/// Until that generation is wired, this returns an empty DB; the dev path above
/// covers `tauri dev`. (M0/M1 always run from the repo, so the dir is found.)
fn embedded() -> ContentDb {
    ContentDb::default()
}

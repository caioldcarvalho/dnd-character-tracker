//! Validate every content file individually, reporting EACH failure (not just the
//! first). Run: cargo run -p rpgman-engine --example validate

use rpgman_engine::*;
use std::path::{Path, PathBuf};

fn files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn check<T: for<'de> serde::Deserialize<'de>>(label: &str, dir: &Path) -> (usize, usize) {
    let (mut ok, mut bad) = (0, 0);
    for path in files(dir) {
        let text = std::fs::read_to_string(&path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        // A file may hold one object or an array of definitions.
        let res: Result<(), String> = if text.trim_start().starts_with('[') {
            serde_json::from_str::<Vec<T>>(&text).map(|_| ()).map_err(|e| e.to_string())
        } else {
            serde_json::from_str::<T>(&text).map(|_| ()).map_err(|e| e.to_string())
        };
        match res {
            Ok(()) => {
                ok += 1;
                println!("  ok   {label}/{name}");
            }
            Err(e) => {
                bad += 1;
                println!("  FAIL {label}/{name}\n         {e}");
            }
        }
    }
    (ok, bad)
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    let mut total_bad = 0;
    for label in ["classes", "subclasses", "species", "backgrounds", "feats"] {
        let dir = root.join(label);
        println!("== {label} ==");
        let (_ok, bad) = match label {
            "classes" => check::<ClassDef>(label, &dir),
            "subclasses" => check::<SubclassDef>(label, &dir),
            "species" => check::<SpeciesDef>(label, &dir),
            "backgrounds" => check::<BackgroundDef>(label, &dir),
            "feats" => check::<Feature>(label, &dir),
            _ => (0, 0),
        };
        total_bad += bad;
    }
    println!("\n{total_bad} file(s) failed to parse.");
    std::process::exit(if total_bad == 0 { 0 } else { 1 });
}

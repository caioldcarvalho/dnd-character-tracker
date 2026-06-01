//! The content baked into the binary (what the packaged app ships) must match
//! the on-disk `content/` directory exactly. Guards against a release that ships
//! with missing or stale classes.

use rpgman_engine::ContentDb;
use std::path::Path;

#[test]
fn embedded_content_matches_the_repo() {
    let disk = ContentDb::load_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content").as_path())
        .expect("content/ should load from disk");
    let embedded = ContentDb::embedded().expect("embedded content should parse");

    assert_eq!(
        embedded.classes.len(),
        disk.classes.len(),
        "embedded classes ({}) != disk ({})",
        embedded.classes.len(),
        disk.classes.len()
    );
    assert!(embedded.classes.len() >= 12, "expected all 12 classes embedded");
    assert_eq!(embedded.subclasses.len(), disk.subclasses.len());
    assert_eq!(embedded.species.len(), disk.species.len());
    assert_eq!(embedded.backgrounds.len(), disk.backgrounds.len());
    assert_eq!(embedded.feats.len(), disk.feats.len());

    // The whole DB should be byte-for-byte identical.
    assert_eq!(embedded, disk, "embedded content must equal on-disk content");
}

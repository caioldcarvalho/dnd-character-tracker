//! Smoke test for the command data path (no GUI/display needed): load the real
//! content, build the bundled sample character, and confirm `compute`/`explain`
//! produce sane results. This is the headless proof that M1's pipeline works.

use rpgman_engine::{compute, explain, Ability, Catalog, CharacterSheet, ContentDb, StatId};
use std::path::Path;

fn content() -> ContentDb {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../content");
    ContentDb::load_dir(&root).expect("content/ should load")
}

/// The same sample the frontend bundles (Aldric, Fighter 11 Battle Master).
fn sample() -> CharacterSheet {
    let mut s = CharacterSheet::new("Aldric")
        .with_species("human")
        .with_background("soldier")
        .with_class("fighter", 11, Some("battle-master"))
        .with_ability(Ability::Str, 16)
        .with_ability(Ability::Dex, 14)
        .with_ability(Ability::Con, 16)
        .with_ability(Ability::Int, 10)
        .with_ability(Ability::Wis, 12)
        .with_ability(Ability::Cha, 13)
        .with_choice("background-abilities", &["str", "str", "con"])
        .with_choice("fighter-fighting-style", &["defense"])
        .with_choice("battle-master-student-of-war", &["history"])
        .with_choice("human-skill", &["perception"]);
    s.equipment.armor = Some(rpgman_engine::ArmorItem {
        name: "Chain Mail".into(),
        base_ac: 16,
        kind: rpgman_engine::ArmorKind::Heavy,
        dex_cap: Some(0),
    });
    s.equipment.shield = true;
    s
}

#[test]
fn catalog_lists_all_content() {
    let cat = Catalog::from_content(&content());
    assert!(cat.classes.len() >= 12, "expected all classes, got {}", cat.classes.len());
    assert!(cat.species.iter().any(|s| s.id == "human"));
    assert!(cat.backgrounds.iter().any(|b| b.id == "soldier"));
}

#[test]
fn compute_produces_a_sane_sheet() {
    let cc = compute(&sample(), &content());
    assert!(cc.errors.is_empty(), "compute errors: {:?}", cc.errors);
    assert_eq!(cc.level, 11);
    assert_eq!(cc.proficiency_bonus, 4);
    // Chain Mail 16 + shield 2 + Defense style 1 = 19.
    assert_eq!(cc.armor_class.total, 19);
    assert!(cc.max_hp.total > 100, "Fighter 11 should have >100 HP");
    // The character has a weapon, resources, and features for the UI to render.
    assert!(!cc.resources.is_empty(), "Battle Master should have superiority dice");
    assert!(!cc.features.is_empty());
}

#[test]
fn explain_returns_a_breakdown_for_the_inspector() {
    let bd = explain(&sample(), &content(), &StatId::ArmorClass);
    assert_eq!(bd.total, 19);
    // The breakdown must carry the named contributions the Inspector renders.
    assert!(bd.lines.iter().any(|l| l.note.as_deref() == Some("Chain Mail")));
    assert!(bd.lines.iter().any(|l| l.note.as_deref() == Some("Shield")));
    assert!(bd.lines.iter().any(|l| l.note.as_deref() == Some("Defense")));
    // The unarmored base (10+DEX) lost the max → present but not applied.
    assert!(
        bd.lines.iter().any(|l| !l.applied),
        "expected a rejected contribution line (unarmored base)"
    );
}

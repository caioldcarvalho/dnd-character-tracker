//! End-to-end: a data-driven Wizard 5 (Evoker), exercising the spellcasting
//! subsystem — slot table, save DC / attack bonus, prepared count, expertise.

use rpgman_engine::*;
use std::path::Path;

fn content() -> ContentDb {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    ContentDb::load_dir(&root).expect("content/ should load")
}

fn mira() -> CharacterSheet {
    CharacterSheet::new("Mira")
        .with_class("wizard", 5, Some("evoker"))
        .with_ability(Ability::Int, 16)
        .with_ability(Ability::Dex, 14)
        .with_ability(Ability::Con, 14)
        .with_ability(Ability::Wis, 12)
        .with_choice("wizard-skills", &["arcana", "investigation"])
}

#[test]
fn wizard5_evoker_computes_from_data() {
    let cc = compute(&mira(), &content());

    assert_eq!(cc.level, 5);
    assert_eq!(cc.proficiency_bonus, 3);

    // HP: L1 max d6 (6) + 4×avg d6 (16) + CON(+2)×5 (10) = 32.
    assert_eq!(cc.max_hp.total, 32);

    // INT save proficient: mod 3 + prof 3 = 6.
    let int_save = cc.saves.iter().find(|s| s.ability == Ability::Int).unwrap();
    assert_eq!(int_save.test.total, 6);
    assert!(int_save.proficient);
}

#[test]
fn full_caster_slot_table() {
    let cc = compute(&mira(), &content());
    let slot = |lvl: u8| cc.spell_slots.iter().find(|s| s.level == lvl).map(|s| s.max);
    // Full caster at level 5: 4 / 3 / 2.
    assert_eq!(slot(1), Some(4));
    assert_eq!(slot(2), Some(3));
    assert_eq!(slot(3), Some(2));
    assert_eq!(slot(4), None);
}

#[test]
fn spell_save_dc_attack_and_prepared_count() {
    let cc = compute(&mira(), &content());
    let sc = cc.spellcasting.iter().find(|s| s.source == "wizard").unwrap();
    assert_eq!(sc.ability, Ability::Int);
    assert_eq!(sc.save_dc, 14, "8 + prof 3 + INT 3");
    assert_eq!(sc.attack_bonus, 6, "prof 3 + INT 3");
    assert_eq!(sc.prepared, Some(9), "wizard prepared table at level 5");
}

#[test]
fn dc_breakdown_is_traceable() {
    let sheet = mira();
    let db = content();
    let bd = explain(&sheet, &db, &StatId::SpellSaveDc(CastingSource::new("wizard")));
    assert_eq!(bd.total, 14);
    // The breakdown names its three parts.
    assert!(bd.lines.iter().any(|l| l.note.as_deref() == Some("Base 8")));
    assert!(bd.lines.iter().any(|l| l.note.as_deref() == Some("Proficiency")));
    assert!(bd.lines.iter().any(|l| l.note.as_deref() == Some("INT")));
}

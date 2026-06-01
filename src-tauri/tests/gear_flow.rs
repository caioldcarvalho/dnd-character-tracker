//! M4 gear: equipping armor/shield/weapons feeds the engine, and notes persist
//! through serialization. Mirrors the store's gear mutations. Headless.

use rpgman_engine::{compute, ArmorItem, ArmorKind, CharacterSheet, ContentDb, Dice, WeaponInstance, WeaponKind};
use std::path::Path;

fn content() -> ContentDb {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../content");
    ContentDb::load_dir(&root).expect("content/ should load")
}

fn fighter() -> CharacterSheet {
    CharacterSheet::new("Gar")
        .with_class("fighter", 5, Some("battle-master"))
        .with_ability(rpgman_engine::Ability::Str, 16)
        .with_ability(rpgman_engine::Ability::Dex, 14)
        .with_ability(rpgman_engine::Ability::Con, 14)
}

#[test]
fn equipping_armor_and_shield_changes_ac() {
    let db = content();
    let mut s = fighter();

    // Unarmored: 10 + DEX(2) = 12.
    assert_eq!(compute(&s, &db).armor_class.total, 12);

    // store.setArmor(Chain Mail): heavy, base 16, no DEX → 16.
    s.equipment.armor = Some(ArmorItem {
        name: "Chain Mail".into(),
        base_ac: 16,
        kind: ArmorKind::Heavy,
        dex_cap: Some(0),
    });
    assert_eq!(compute(&s, &db).armor_class.total, 16);

    // store.toggleShield(): +2 → 18.
    s.equipment.shield = true;
    assert_eq!(compute(&s, &db).armor_class.total, 18);

    // store.setArmor(null): back to unarmored, but shield still applies → 12+2=14.
    s.equipment.armor = None;
    assert_eq!(compute(&s, &db).armor_class.total, 14);
}

#[test]
fn adding_a_weapon_yields_computed_attack_and_damage() {
    let db = content();
    let mut s = fighter();
    // store.addWeapon(...): a +1 longsword.
    s.weapons.push(WeaponInstance {
        name: "Longsword +1".into(),
        kind: WeaponKind::Melee,
        damage: Dice::new(1, 8),
        damage_type: "slashing".into(),
        finesse: false,
        two_handed: false,
        magic_bonus: 1,
        proficient: true,
        mastery: Some("sap".into()),
    });
    let cc = compute(&s, &db);
    let w = cc.weapons.iter().find(|w| w.name == "Longsword +1").unwrap();
    // prof 3 + STR 3 + magic 1 = 7.
    assert_eq!(w.attack_bonus, 7);
    // STR 3 + magic 1 = 4.
    assert_eq!(w.damage_bonus, 4);
    assert_eq!(w.mastery.as_deref(), Some("sap"));
}

#[test]
fn notes_round_trip_through_serialization() {
    let mut s = fighter();
    s.notes = "Owes the guild 50gp. Afraid of fire.".into();
    let json = serde_json::to_string(&s).unwrap();
    let back: CharacterSheet = serde_json::from_str(&json).unwrap();
    assert_eq!(back.notes, "Owes the guild 50gp. Afraid of fire.");
}

#[test]
fn weapon_attack_breakdown_shows_proficiency_and_ability() {
    use rpgman_engine::{Dice, WeaponInstance, WeaponKind};
    let db = content();
    let mut s = fighter(); // STR 16 (+3), fighter 5 (prof +3)
    s.weapons.push(WeaponInstance {
        name: "Longsword".into(),
        kind: WeaponKind::Melee,
        damage: Dice::new(1, 8),
        damage_type: "slashing".into(),
        finesse: false,
        two_handed: false,
        magic_bonus: 2,
        proficient: true,
        mastery: None,
    });
    let cc = compute(&s, &db);
    let w = cc.weapons.iter().find(|w| w.name == "Longsword").unwrap();
    // Attack = STR 3 + prof 3 + magic 2 = 8.
    assert_eq!(w.attack_bonus, 8);
    let ab = &w.attack_breakdown;
    assert_eq!(ab.total, 8);
    assert!(ab.lines.iter().any(|l| l.source == "STR" && l.value == 3), "STR line present");
    assert!(ab.lines.iter().any(|l| l.source == "Proficiency" && l.value == 3), "Proficiency line present");
    assert!(ab.lines.iter().any(|l| l.source == "Magic weapon" && l.value == 2), "magic line present");
    // Damage = STR 3 + magic 2 = 5 (NO proficiency).
    let dm = &w.damage_breakdown;
    assert_eq!(dm.total, 5);
    assert!(dm.lines.iter().any(|l| l.source == "STR" && l.value == 3));
    assert!(!dm.lines.iter().any(|l| l.source == "Proficiency"), "proficiency must NOT add to damage");
}

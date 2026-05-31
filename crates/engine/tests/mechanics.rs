//! Deeper mechanics: weapon attack/damage + fighting styles, weapon mastery
//! counts, Rage as a toggle, Unarmored Defense, and multiclass spell slots.

use rpgman_engine::*;
use std::collections::BTreeMap;
use std::path::Path;

fn content() -> ContentDb {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    ContentDb::load_dir(&root).expect("content/ should load")
}

fn weapon(name: &str, kind: WeaponKind, sides: u8) -> WeaponInstance {
    WeaponInstance {
        name: name.into(),
        kind,
        damage: Dice::new(1, sides),
        damage_type: "slashing".into(),
        finesse: false,
        two_handed: false,
        magic_bonus: 0,
        proficient: true,
        mastery: None,
    }
}

#[test]
fn archery_adds_to_ranged_attacks_only() {
    let mut s = CharacterSheet::new("Archer")
        .with_class("fighter", 5, None)
        .with_ability(Ability::Str, 16)
        .with_ability(Ability::Dex, 14)
        .with_choice("fighter-fighting-style", &["archery"]);
    s.weapons.push(weapon("Longbow", WeaponKind::Ranged, 8));
    s.weapons.push(weapon("Longsword", WeaponKind::Melee, 8));
    let cc = compute(&s, &content());

    let bow = cc.weapons.iter().find(|w| w.name == "Longbow").unwrap();
    assert_eq!(bow.attack_bonus, 3 + 2 + 2, "prof 3 + DEX 2 + Archery 2");
    assert_eq!(bow.damage_bonus, 2, "DEX 2");

    let sword = cc.weapons.iter().find(|w| w.name == "Longsword").unwrap();
    assert_eq!(sword.attack_bonus, 3 + 3, "prof 3 + STR 3, Archery does not apply");
    assert_eq!(sword.damage_bonus, 3);
}

#[test]
fn dueling_adds_to_melee_damage() {
    let mut s = CharacterSheet::new("Duelist")
        .with_class("fighter", 5, None)
        .with_ability(Ability::Str, 16)
        .with_choice("fighter-fighting-style", &["dueling"]);
    s.weapons.push(weapon("Longsword", WeaponKind::Melee, 8));
    let cc = compute(&s, &content());
    let sword = &cc.weapons[0];
    assert_eq!(sword.attack_bonus, 3 + 3);
    assert_eq!(sword.damage_bonus, 3 + 2, "STR 3 + Dueling 2");
}

#[test]
fn weapon_masteries_known_scales() {
    let s = CharacterSheet::new("F").with_class("fighter", 5, None);
    assert_eq!(compute(&s, &content()).masteries_known, 4, "fighter masteries at L5");
}

#[test]
fn rage_is_a_toggle_that_only_applies_while_active() {
    let mut base = CharacterSheet::new("Grog")
        .with_class("barbarian", 3, Some("any")) // subclass id need not exist for this test
        .with_ability(Ability::Str, 16)
        .with_ability(Ability::Dex, 14)
        .with_ability(Ability::Con, 14);
    base.weapons.push(weapon("Greataxe", WeaponKind::Melee, 12));

    // Rage OFF.
    let off = compute(&base, &content());
    assert_eq!(off.weapons[0].damage_bonus, 3, "STR only");
    let str_save_off = off.saves.iter().find(|s| s.ability == Ability::Str).unwrap();
    assert_eq!(str_save_off.test.adv, AdvState::Flat);
    assert!(off.effects.iter().any(|e| e.id == "rage" && !e.active));

    // Rage ON.
    let mut raging = base.clone();
    raging.active_effects.push("rage".into());
    let on = compute(&raging, &content());
    assert_eq!(on.weapons[0].damage_bonus, 3 + 2, "STR 3 + Rage 2");
    let str_save_on = on.saves.iter().find(|s| s.ability == Ability::Str).unwrap();
    assert_eq!(str_save_on.test.adv, AdvState::Advantage, "Rage grants advantage on STR saves");
    assert!(on.effects.iter().any(|e| e.id == "rage" && e.active));
}

#[test]
fn unarmored_defense_uses_dex_plus_con() {
    let s = CharacterSheet::new("Grog")
        .with_class("barbarian", 3, Some("any"))
        .with_ability(Ability::Dex, 14)
        .with_ability(Ability::Con, 14);
    // No armor → Unarmored Defense base 10 + DEX(2) + CON(2) = 14, beats plain 10 + DEX.
    assert_eq!(compute(&s, &content()).armor_class.total, 14);
}

#[test]
fn multiclass_spell_slots_combine_caster_levels() {
    // Built in Rust: a full caster + a half caster.
    let mk = |id: &str, prog: Progression| ClassDef {
        id: ClassId::new(id),
        name: id.to_string(),
        hit_die: 8,
        saving_throws: vec![],
        spellcasting: Some(CasterSpec {
            ability: Ability::Cha,
            progression: prog,
            preparation: Preparation::Prepared,
            prepared_per_level: None,
        }),
        subclass_level: 3,
        levels: BTreeMap::new(),
    };
    let content = ContentDb::default()
        .with_class(mk("wizard", Progression::Full))
        .with_class(mk("paladin", Progression::Half));

    // Wizard 2 (full) + Paladin 4 (half = 2) → caster level 4 → slots 4 / 3.
    let s = CharacterSheet::new("MC")
        .with_class("wizard", 2, None)
        .with_class("paladin", 4, None)
        .with_ability(Ability::Cha, 16);
    let cc = compute(&s, &content);
    let slot = |lvl: u8| cc.spell_slots.iter().find(|x| x.level == lvl).map(|x| x.max);
    assert_eq!(slot(1), Some(4));
    assert_eq!(slot(2), Some(3));
    assert_eq!(slot(3), None);
}

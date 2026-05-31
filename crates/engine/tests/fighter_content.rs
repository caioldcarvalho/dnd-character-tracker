//! End-to-end: load the real rules-as-data files from `content/` and build a
//! Fighter 11 (Battle Master). Proves the engine interprets data with no
//! class-specific code.

use rpgman_engine::*;
use std::path::Path;

fn content() -> ContentDb {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    ContentDb::load_dir(&root).expect("content/ should load")
}

fn aldric() -> CharacterSheet {
    let mut s = CharacterSheet::new("Aldric")
        .with_class("fighter", 11, Some("battle-master"))
        .with_ability(Ability::Str, 16)
        .with_ability(Ability::Dex, 14)
        .with_ability(Ability::Con, 16)
        .with_ability(Ability::Wis, 12)
        .with_choice("fighter-fighting-style", &["defense"])
        .with_choice("battle-master-student-of-war", &["history"]);
    s.equipment.armor = Some(ArmorItem {
        name: "Chain Mail".into(),
        base_ac: 16,
        kind: ArmorKind::Heavy,
        dex_cap: Some(0),
    });
    s
}

#[test]
fn content_loads() {
    let c = content();
    assert!(c.class(&ClassId::new("fighter")).is_some());
    assert!(c.subclass("battle-master").is_some());
    assert!(c.feat("defense").is_some());
}

#[test]
fn fighter11_battlemaster_computes_from_data() {
    let cc = compute(&aldric(), &content());

    assert_eq!(cc.level, 11);
    assert_eq!(cc.proficiency_bonus, 4);

    // HP: L1 max d10 (10) + 10×avg d10 (60) + CON(+3)×11 (33) = 103.
    assert_eq!(cc.max_hp.total, 103);

    // AC: Chain Mail base 16 + Defense fighting style (+1 while armored) = 17.
    assert_eq!(cc.armor_class.total, 17);
    assert!(
        cc.armor_class
            .lines
            .iter()
            .any(|l| l.applied && l.note.as_deref() == Some("Defense")),
        "Defense should be a visible, conditional contribution"
    );

    // STR save: mod 3 + proficiency 4 = 7.
    let str_save = cc.saves.iter().find(|s| s.ability == Ability::Str).unwrap();
    assert_eq!(str_save.test.total, 7);
    assert!(str_save.proficient);

    // Student of War granted History proficiency via a recorded choice.
    let history = cc.skills.iter().find(|s| s.skill == Skill::History).unwrap();
    assert!(history.proficient);
}

#[test]
fn extra_attack_scales_via_a_level_table() {
    let sheet = aldric();
    let db = content();
    // 3 attacks per action at fighter level 11.
    assert_eq!(explain(&sheet, &db, &StatId::AttacksPerAction).total, 3);
}

#[test]
fn resource_pools_are_data_driven() {
    let cc = compute(&aldric(), &content());
    let res = |id: &str| cc.resources.iter().find(|r| r.id == ResourceId::new(id)).unwrap();

    // Superiority Dice: 5 dice (table 3→4, 7→5, 15→6) of d10 (scaling 3→d8, 10→d10, 18→d12).
    let sup = res("superiority-dice");
    assert_eq!(sup.max, 5);
    assert_eq!(sup.die, Some(10));
    assert_eq!(sup.kind, ResourceKind::Dice);

    // Second Wind: 4 uses at level 11 (table 1→2, 4→3, 10→4).
    assert_eq!(res("second-wind").max, 4);
    // Action Surge: 1 use (2 only at level 17).
    assert_eq!(res("action-surge").max, 1);
    // Indomitable: 1 use at level 11 (2 only at level 13).
    assert_eq!(res("indomitable").max, 1);
}

#[test]
fn level_up_choices_are_surfaced_but_subclass_is_satisfied() {
    let cc = compute(&aldric(), &content());

    // Subclass is chosen, so it must NOT be pending.
    assert!(!cc
        .pending_choices
        .iter()
        .any(|p| matches!(p.options, ChoiceOptions::Subclass)));

    // Weapon mastery picks were never recorded → still pending.
    assert!(cc
        .pending_choices
        .iter()
        .any(|p| matches!(p.options, ChoiceOptions::WeaponMastery { .. })));

    // ASIs at 4/6/8 were not taken → pending.
    assert!(cc
        .pending_choices
        .iter()
        .filter(|p| matches!(p.options, ChoiceOptions::AbilityScoreIncrease))
        .count()
        >= 3);

    // The Combat Superiority feature is listed.
    assert!(cc.features.iter().any(|f| f.name == "Combat Superiority"));
}

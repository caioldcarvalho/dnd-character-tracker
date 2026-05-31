//! Validates the "widening" content: species, backgrounds, origin feats, and the
//! Psi Warrior subclass — all as data, all computing through the unchanged engine.

use rpgman_engine::*;
use std::path::Path;

fn content() -> ContentDb {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    ContentDb::load_dir(&root).expect("content/ should load")
}

#[test]
fn all_content_files_parse() {
    let c = content();
    assert!(c.background("soldier").is_some());
    assert!(c.background("sage").is_some());
    assert!(c.species_def("human").is_some());
    assert!(c.species_def("elf").is_some());
    assert!(c.species_def("dwarf").is_some());
    assert!(c.feat("tough").is_some());
    assert!(c.feat("alert").is_some());
    assert!(c.subclass("psi-warrior").is_some());
}

#[test]
fn background_grants_skills_origin_feat_and_an_ability_choice() {
    // Soldier background, ability spread recorded as +2 STR / +1 CON.
    let s = CharacterSheet::new("Recruit")
        .with_class("fighter", 1, None)
        .with_background("soldier")
        .with_ability(Ability::Str, 14)
        .with_ability(Ability::Con, 13)
        .with_choice("background-abilities", &["str", "str", "con"]);
    let cc = compute(&s, &content());

    let str_score = cc.abilities.iter().find(|a| a.ability == Ability::Str).unwrap().score;
    let con_score = cc.abilities.iter().find(|a| a.ability == Ability::Con).unwrap().score;
    assert_eq!(str_score, 16, "Soldier +2 STR");
    assert_eq!(con_score, 14, "Soldier +1 CON");

    assert!(cc.skills.iter().find(|x| x.skill == Skill::Athletics).unwrap().proficient);
    assert!(cc.skills.iter().find(|x| x.skill == Skill::Intimidation).unwrap().proficient);

    assert!(cc.features.iter().any(|f| f.name == "Savage Attacker"));
}

#[test]
fn background_ability_choice_is_pending_when_unrecorded() {
    let s = CharacterSheet::new("Recruit")
        .with_class("fighter", 1, None)
        .with_background("soldier");
    let cc = compute(&s, &content());
    assert!(cc.pending_choices.iter().any(|p| p.key == "background-abilities"));
}

#[test]
fn tough_origin_feat_adds_twice_level_to_hp() {
    let base = CharacterSheet::new("F").with_class("fighter", 5, None).with_ability(Ability::Con, 14);
    let without = compute(&base, &content()).max_hp.total;
    let with = compute(&base.clone().with_feat("tough"), &content()).max_hp.total;
    assert_eq!(with - without, 10, "Tough = +2 × level (5) = +10");
}

#[test]
fn alert_adds_proficiency_bonus_to_initiative() {
    let base = CharacterSheet::new("F").with_class("fighter", 5, None).with_ability(Ability::Dex, 14);
    let without = compute(&base, &content()).initiative.total;
    let with = compute(&base.clone().with_feat("alert"), &content()).initiative.total;
    assert_eq!(with - without, 3, "Alert = +PB (3 at level 5)");
}

#[test]
fn lucky_pool_size_equals_proficiency_bonus() {
    let s = CharacterSheet::new("L").with_class("fighter", 9, None).with_feat("lucky");
    let cc = compute(&s, &content());
    let luck = cc.resources.iter().find(|r| r.id == ResourceId::new("luck-points")).unwrap();
    assert_eq!(luck.max, 4, "PB at level 9 is +4");
}

#[test]
fn dwarven_toughness_adds_one_hp_per_level() {
    let human = CharacterSheet::new("H").with_class("fighter", 5, None).with_species("human").with_ability(Ability::Con, 14);
    let dwarf = CharacterSheet::new("D").with_class("fighter", 5, None).with_species("dwarf").with_ability(Ability::Con, 14);
    let diff = compute(&dwarf, &content()).max_hp.total - compute(&human, &content()).max_hp.total;
    assert_eq!(diff, 5, "Dwarven Toughness = +1 per level");
}

#[test]
fn psi_warrior_energy_dice_use_the_generic_resource_pool() {
    // The user's own character: a Psi Warrior. Energy Dice = 2 × PB, die scales by level.
    // Proves the SAME ResourceDef that powers Battle Master superiority dice handles
    // psionics with ZERO special-casing (the thing svelte-app hardcoded).
    let s = CharacterSheet::new("Kael")
        .with_class("fighter", 12, Some("psi-warrior"))
        .with_ability(Ability::Str, 16)
        .with_ability(Ability::Con, 16)
        .with_ability(Ability::Int, 14);
    let cc = compute(&s, &content());

    let energy = cc.resources.iter().find(|r| r.id == ResourceId::new("energy-dice")).unwrap();
    assert_eq!(energy.max, 8, "2 × PB (4 at level 12) = 8 dice");
    assert_eq!(energy.die, Some(10), "d10 at fighter level 11+");
    assert_eq!(energy.kind, ResourceKind::Dice);

    let bd = explain(&s, &content(), &StatId::ResourceMax(ResourceId::new("energy-dice")));
    assert_eq!(bd.total, 8, "pool max is itself traceable");

    assert!(cc.features.iter().any(|f| f.name == "Psionic Power"));
    assert!(cc.features.iter().any(|f| f.name == "Guarded Mind"));
}

#[test]
fn human_grants_skill_and_origin_feat_choices() {
    let s = CharacterSheet::new("H")
        .with_class("fighter", 1, None)
        .with_species("human")
        .with_choice("human-skill", &["perception"])
        .with_choice("human-origin-feat", &["tough"]);
    let cc = compute(&s, &content());
    assert!(cc.skills.iter().find(|x| x.skill == Skill::Perception).unwrap().proficient);
    assert!(cc.features.iter().any(|f| f.name == "Tough"));
}

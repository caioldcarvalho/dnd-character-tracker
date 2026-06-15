//! Tests that active conditions apply the correct advantage/disadvantage on the
//! character's own d20 rolls, and that they compose correctly with each other
//! and with exhaustion.

use rpgman_engine::*;
use std::collections::BTreeMap;

fn fighter() -> ClassDef {
    ClassDef {
        id: ClassId::new("fighter"),
        name: "Fighter".into(),
        hit_die: 10,
        saving_throws: vec![Ability::Str, Ability::Con],
        spellcasting: None,
        subclass_level: 3,
        levels: BTreeMap::new(),
    }
}

fn base_content() -> ContentDb {
    ContentDb::default().with_class(fighter())
}

fn fighter5() -> CharacterSheet {
    CharacterSheet::new("Test")
        .with_class("fighter", 5, None)
        .with_ability(Ability::Str, 16)
        .with_ability(Ability::Dex, 14)
        .with_ability(Ability::Con, 16)
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

// Helper: rebuild EvalCtx and query the weapon attack d20 channel directly.
fn weapon_attack_adv(sheet: &CharacterSheet, content: &ContentDb, kind: WeaponKind) -> AdvState {
    let built = build(sheet, content);
    let ctx = EvalCtx::new(&built);
    ctx.d20(&StatId::WeaponAttackBonus(kind)).adv
}

fn skill_adv(cc: &ComputedCharacter, sk: Skill) -> AdvState {
    cc.skills.iter().find(|s| s.skill == sk).unwrap().test.adv
}

fn save_adv(cc: &ComputedCharacter, a: Ability) -> AdvState {
    cc.saves.iter().find(|s| s.ability == a).unwrap().test.adv
}

// ---- Poisoned ---------------------------------------------------------------

#[test]
fn poisoned_gives_disadvantage_on_melee_attack() {
    let mut s = fighter5();
    s.weapons.push(weapon("Longsword", WeaponKind::Melee, 8));

    // Clean character: no disadvantage.
    let adv_clean = weapon_attack_adv(&s, &base_content(), WeaponKind::Melee);
    assert_eq!(adv_clean, AdvState::Flat, "clean character has flat melee attack");

    // Poisoned: disadvantage.
    s.conditions.push("poisoned".into());
    let adv_poisoned = weapon_attack_adv(&s, &base_content(), WeaponKind::Melee);
    assert_eq!(adv_poisoned, AdvState::Disadvantage, "poisoned → disadvantage on melee attack");
}

#[test]
fn poisoned_gives_disadvantage_on_ranged_attack() {
    let mut s = fighter5();
    s.conditions.push("poisoned".into());
    let adv = weapon_attack_adv(&s, &base_content(), WeaponKind::Ranged);
    assert_eq!(adv, AdvState::Disadvantage, "poisoned → disadvantage on ranged attack");
}

#[test]
fn poisoned_gives_disadvantage_on_ability_checks() {
    let mut s = fighter5();
    s.conditions.push("poisoned".into());
    let cc = compute(&s, &base_content());
    // All skills (which are the canonical ability-check stats) should have disadvantage.
    for sk in Skill::ALL {
        assert_eq!(
            skill_adv(&cc, sk),
            AdvState::Disadvantage,
            "poisoned → disadvantage on {:?} check",
            sk
        );
    }
}

#[test]
fn poisoned_does_not_affect_saving_throws() {
    let mut s = fighter5();
    s.conditions.push("poisoned".into());
    let cc = compute(&s, &base_content());
    // Poisoned only affects attacks + ability checks, not saves.
    assert_eq!(
        save_adv(&cc, Ability::Str),
        AdvState::Flat,
        "poisoned does not affect STR saves"
    );
}

// ---- Frightened -------------------------------------------------------------

#[test]
fn frightened_gives_disadvantage_on_attacks_and_checks() {
    let mut s = fighter5();
    s.conditions.push("frightened".into());

    let melee = weapon_attack_adv(&s, &base_content(), WeaponKind::Melee);
    assert_eq!(melee, AdvState::Disadvantage, "frightened → dis on melee attacks");

    let cc = compute(&s, &base_content());
    assert_eq!(
        skill_adv(&cc, Skill::Athletics),
        AdvState::Disadvantage,
        "frightened → dis on ability checks"
    );
}

// ---- Prone ------------------------------------------------------------------

#[test]
fn prone_gives_disadvantage_on_attacks() {
    let mut s = fighter5();
    s.conditions.push("prone".into());

    assert_eq!(
        weapon_attack_adv(&s, &base_content(), WeaponKind::Melee),
        AdvState::Disadvantage,
        "prone → dis on melee"
    );
    assert_eq!(
        weapon_attack_adv(&s, &base_content(), WeaponKind::Ranged),
        AdvState::Disadvantage,
        "prone → dis on ranged"
    );
}

#[test]
fn prone_does_not_affect_ability_checks() {
    let mut s = fighter5();
    s.conditions.push("prone".into());
    let cc = compute(&s, &base_content());
    assert_eq!(
        skill_adv(&cc, Skill::Athletics),
        AdvState::Flat,
        "prone does not affect ability checks"
    );
}

// ---- Restrained -------------------------------------------------------------

#[test]
fn restrained_gives_disadvantage_on_attacks_and_dex_save() {
    let mut s = fighter5();
    s.conditions.push("restrained".into());

    let cc = compute(&s, &base_content());
    assert_eq!(
        weapon_attack_adv(&s, &base_content(), WeaponKind::Melee),
        AdvState::Disadvantage,
        "restrained → dis on attacks"
    );
    assert_eq!(
        save_adv(&cc, Ability::Dex),
        AdvState::Disadvantage,
        "restrained → dis on DEX save"
    );
    // STR save not affected.
    assert_eq!(
        save_adv(&cc, Ability::Str),
        AdvState::Flat,
        "restrained does not affect STR save"
    );
}

// ---- Blinded ----------------------------------------------------------------

#[test]
fn blinded_gives_disadvantage_on_attacks() {
    let mut s = fighter5();
    s.conditions.push("blinded".into());
    assert_eq!(
        weapon_attack_adv(&s, &base_content(), WeaponKind::Melee),
        AdvState::Disadvantage,
        "blinded → dis on melee"
    );
}

// ---- Invisible --------------------------------------------------------------

#[test]
fn invisible_gives_advantage_on_attacks() {
    let mut s = fighter5();
    s.conditions.push("invisible".into());
    assert_eq!(
        weapon_attack_adv(&s, &base_content(), WeaponKind::Melee),
        AdvState::Advantage,
        "invisible → adv on melee"
    );
    assert_eq!(
        weapon_attack_adv(&s, &base_content(), WeaponKind::Ranged),
        AdvState::Advantage,
        "invisible → adv on ranged"
    );
}

// ---- Paralyzed / Stunned / Unconscious / Petrified --------------------------

#[test]
fn paralyzed_gives_disadvantage_on_str_and_dex_saves() {
    for cond in &["paralyzed", "stunned", "unconscious", "petrified"] {
        let mut s = fighter5();
        s.conditions.push(cond.to_string());
        let cc = compute(&s, &base_content());
        assert_eq!(
            save_adv(&cc, Ability::Str),
            AdvState::Disadvantage,
            "{} → dis on STR save",
            cond
        );
        assert_eq!(
            save_adv(&cc, Ability::Dex),
            AdvState::Disadvantage,
            "{} → dis on DEX save",
            cond
        );
        // CON save should not be affected.
        assert_eq!(
            save_adv(&cc, Ability::Con),
            AdvState::Flat,
            "{} does not affect CON save",
            cond
        );
    }
}

// ---- Composition: adv + dis cancel -----------------------------------------

#[test]
fn invisible_and_blinded_cancel_to_flat() {
    // Invisible gives adv on attacks, blinded gives dis — they cancel.
    let mut s = fighter5();
    s.conditions.push("invisible".into());
    s.conditions.push("blinded".into());
    assert_eq!(
        weapon_attack_adv(&s, &base_content(), WeaponKind::Melee),
        AdvState::Flat,
        "invisible + blinded = flat (adv cancels dis)"
    );
}

#[test]
fn conditions_and_exhaustion_compose_correctly() {
    // Exhaustion adds a numeric penalty; poisoned adds disadvantage on attacks.
    // Both should be present simultaneously.
    let mut s = fighter5();
    s.conditions.push("poisoned".into());
    s.exhaustion = 2;
    let cc = compute(&s, &base_content());

    // Poisoned: disadvantage on attacks.
    assert_eq!(
        weapon_attack_adv(&s, &base_content(), WeaponKind::Melee),
        AdvState::Disadvantage,
        "poisoned + exhaustion: still disadvantage on attacks"
    );
    // Exhaustion: −4 numeric penalty on saves (−2 per level).
    let str_save = cc.saves.iter().find(|s| s.ability == Ability::Str).unwrap();
    // STR save base: mod 3 + prof 3 = 6, exhaustion 2 = −4 → 2.
    assert_eq!(str_save.test.total, 2, "exhaustion −4 still applies numerically");
}

// ---- ComputedCharacter exposes conditions -----------------------------------

#[test]
fn computed_character_exposes_active_conditions() {
    let mut s = fighter5();
    s.conditions.push("poisoned".into());
    s.conditions.push("prone".into());
    let cc = compute(&s, &base_content());
    assert!(cc.conditions.contains(&"poisoned".to_string()));
    assert!(cc.conditions.contains(&"prone".to_string()));
    assert_eq!(cc.conditions.len(), 2);
}

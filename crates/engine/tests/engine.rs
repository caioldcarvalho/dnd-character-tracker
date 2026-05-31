//! Engine math tests using Rust-constructed fixtures (no content files yet).

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

/// Fighter 5, CON 16, STR 16, DEX 14.
fn fighter5() -> CharacterSheet {
    CharacterSheet::new("Test")
        .with_class("fighter", 5, None)
        .with_ability(Ability::Str, 16)
        .with_ability(Ability::Dex, 14)
        .with_ability(Ability::Con, 16)
}

fn save(c: &ComputedCharacter, a: Ability) -> &SaveView {
    c.saves.iter().find(|s| s.ability == a).unwrap()
}
fn skill(c: &ComputedCharacter, s: Skill) -> &SkillView {
    c.skills.iter().find(|x| x.skill == s).unwrap()
}

#[test]
fn ability_modifiers_and_proficiency_bonus() {
    let c = compute(&fighter5(), &base_content());
    let str_mod = c.abilities.iter().find(|a| a.ability == Ability::Str).unwrap().modifier;
    assert_eq!(str_mod, 3, "STR 16 → +3");
    assert_eq!(c.proficiency_bonus, 3, "level 5 → +3 proficiency");
    assert_eq!(c.level, 5);
}

#[test]
fn hp_is_the_sum_of_its_contributions() {
    // L1 max d10 (10) + 4×avg d10 (24) + CON×5 (15) = 49.
    let c = compute(&fighter5(), &base_content());
    assert_eq!(c.max_hp.total, 49);

    // Tough feat adds 2 × character level = +10 → 59, traceable as its own line.
    let content = base_content().with_feat(
        Feature::new("tough", "Tough").with_contribution(
            Contribution::add(
                StatId::MaxHitPoints,
                ValueExpr::scaled(ValueExpr::lit(2), ValueExpr::stat(StatId::CharacterLevel)),
            )
            .note("Tough"),
        ),
    );
    let c = compute(&fighter5().with_feat("tough"), &content);
    assert_eq!(c.max_hp.total, 59);
    assert!(c
        .max_hp
        .lines
        .iter()
        .any(|l| l.applied && l.note.as_deref() == Some("Tough") && l.value == 10));
}

#[test]
fn armor_class_takes_the_max_of_competing_bases_then_adds_shield() {
    let content = base_content();

    // Unarmored: 10 + DEX(2) = 12.
    assert_eq!(compute(&fighter5(), &content).armor_class.total, 12);

    // Chain mail (heavy, base 16, no DEX): max(12, 16) = 16.
    let mut s = fighter5();
    s.equipment.armor = Some(ArmorItem {
        name: "Chain Mail".into(),
        base_ac: 16,
        kind: ArmorKind::Heavy,
        dex_cap: Some(0),
    });
    assert_eq!(compute(&s, &content).armor_class.total, 16);

    // + shield: 16 + 2 = 18.
    s.equipment.shield = true;
    assert_eq!(compute(&s, &content).armor_class.total, 18);

    // Leather (light, base 11, full DEX): max(12, 11+2=13) = 13.
    let mut s2 = fighter5();
    s2.equipment.armor = Some(ArmorItem {
        name: "Leather".into(),
        base_ac: 11,
        kind: ArmorKind::Light,
        dex_cap: None,
    });
    assert_eq!(compute(&s2, &content).armor_class.total, 13);
}

#[test]
fn saving_throws_apply_proficiency_and_exhaustion() {
    let content = base_content();
    let c = compute(&fighter5(), &content);
    assert_eq!(save(&c, Ability::Str).test.total, 6, "STR mod 3 + prof 3");
    assert!(save(&c, Ability::Str).proficient);
    assert_eq!(save(&c, Ability::Con).test.total, 6);
    assert_eq!(save(&c, Ability::Dex).test.total, 2, "DEX mod only (not proficient)");
    assert!(!save(&c, Ability::Dex).proficient);

    // Exhaustion 2 (2024): −2 per level on every d20 test.
    let mut s = fighter5();
    s.exhaustion = 2;
    let c = compute(&s, &content);
    assert_eq!(save(&c, Ability::Str).test.total, 6 - 4);
}

#[test]
fn skill_bonus_and_expertise() {
    let content = base_content();
    let c = compute(&fighter5(), &content);
    // Athletics (STR) with no proficiency = STR mod 3.
    assert_eq!(skill(&c, Skill::Athletics).bonus, 3);

    // A feature granting Athletics expertise stacks proficiency twice (3 + 3 + 3 = 9).
    let content = content.with_feat(
        Feature::new("athlete", "Athlete").with_proficiency(ProficiencyGrant::Skill {
            skill: Skill::Athletics,
            expertise: true,
        }),
    );
    let c = compute(&fighter5().with_feat("athlete"), &content);
    let a = skill(&c, Skill::Athletics);
    assert!(a.proficient && a.expertise);
    assert_eq!(a.bonus, 3 + 3 + 3);
}

#[test]
fn advantage_cancels_disadvantage() {
    // Advantage only → Advantage.
    let content = base_content().with_feat(
        Feature::new("blessed", "Blessed").with_contribution(Contribution::d20(
            StatId::SavingThrow(Ability::Str),
            D20Effect::Advantage,
        )),
    );
    let c = compute(&fighter5().with_feat("blessed"), &content);
    assert_eq!(save(&c, Ability::Str).test.adv, AdvState::Advantage);

    // Advantage + Disadvantage → Flat (they don't count up).
    let content = content.with_feat(
        Feature::new("cursed", "Cursed").with_contribution(Contribution::d20(
            StatId::SavingThrow(Ability::Str),
            D20Effect::Disadvantage,
        )),
    );
    let c = compute(&fighter5().with_feat("blessed").with_feat("cursed"), &content);
    assert_eq!(save(&c, Ability::Str).test.adv, AdvState::Flat);
}

#[test]
fn dependency_cycles_are_reported_not_panicked() {
    // A contribution that references the very stat it targets.
    let content = base_content().with_feat(
        Feature::new("ouroboros", "Ouroboros").with_contribution(Contribution::add(
            StatId::AbilityScore(Ability::Str),
            ValueExpr::stat(StatId::AbilityScore(Ability::Str)),
        )),
    );
    let c = compute(&fighter5().with_feat("ouroboros"), &content);
    assert!(
        c.errors.iter().any(|e| e.stat == StatId::AbilityScore(Ability::Str)),
        "expected a cycle error, got {:?}",
        c.errors
    );
    // STR score still resolves to its base (cycle contributes 0), no panic.
    let str_score = c.abilities.iter().find(|a| a.ability == Ability::Str).unwrap().score;
    assert_eq!(str_score, 16);
}

#[test]
fn pending_subclass_choice_surfaces_at_level_3() {
    let c = compute(&fighter5(), &base_content());
    assert!(
        c.pending_choices.iter().any(|p| matches!(p.options, ChoiceOptions::Subclass)),
        "a level-5 Fighter without a subclass should be prompted to pick one"
    );
}

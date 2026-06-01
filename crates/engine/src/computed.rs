//! Public entry points. [`compute`] interprets a sheet + content into a fully
//! derived [`ComputedCharacter`] (with a breakdown for the headline stats and a
//! resolved d20 test for every save/skill). [`explain`] returns the breakdown for
//! any single stat.

use crate::build::{build, PendingChoice, ResolvedFeature};
use crate::content::ContentDb;
use crate::eval::{D20Test, EvalCtx, StatBreakdown};
use crate::error::EvalError;
use crate::ids::{Ability, MovementKind, ResourceId, Skill, SlotLevel, StatId, WeaponKind};
use crate::resource::{HitDiePool, Recharge, ResourceKind};
use crate::sheet::CharacterSheet;
use crate::value::Dice;
use serde::Serialize;

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct AbilityView {
    pub ability: Ability,
    pub score: i32,
    pub modifier: i32,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct SaveView {
    pub ability: Ability,
    pub proficient: bool,
    pub test: D20Test,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct SkillView {
    pub skill: Skill,
    pub ability: Ability,
    pub proficient: bool,
    pub expertise: bool,
    pub bonus: i32,
    pub test: D20Test,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct PassiveView {
    pub skill: Skill,
    pub value: i32,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct SpeedView {
    pub kind: MovementKind,
    pub value: i32,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct ResourceView {
    pub id: ResourceId,
    pub name: String,
    pub kind: ResourceKind,
    pub current: i32,
    pub max: i32,
    pub die: Option<u8>,
    pub recharge: Recharge,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct SpellSlotView {
    pub level: u8,
    pub max: i32,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct SpellcastingView {
    pub source: String,
    pub ability: Ability,
    pub save_dc: i32,
    pub attack_bonus: i32,
    pub prepared: Option<i32>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct WeaponView {
    pub name: String,
    pub kind: WeaponKind,
    pub attack_bonus: i32,
    pub damage: Dice,
    pub damage_bonus: i32,
    pub damage_type: String,
    pub mastery: Option<String>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct EffectView {
    pub id: String,
    pub name: String,
    pub active: bool,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize)]
pub struct ComputedCharacter {
    pub name: String,
    pub level: i32,
    pub proficiency_bonus: i32,
    pub max_hp: StatBreakdown,
    pub current_hp: i32,
    pub temp_hp: i32,
    pub armor_class: StatBreakdown,
    pub initiative: StatBreakdown,
    pub speeds: Vec<SpeedView>,
    pub abilities: Vec<AbilityView>,
    pub saves: Vec<SaveView>,
    pub skills: Vec<SkillView>,
    pub passives: Vec<PassiveView>,
    pub resources: Vec<ResourceView>,
    pub spell_slots: Vec<SpellSlotView>,
    pub spellcasting: Vec<SpellcastingView>,
    pub weapons: Vec<WeaponView>,
    pub masteries_known: i32,
    pub effects: Vec<EffectView>,
    pub concentration: Option<String>,
    pub hit_dice: Vec<HitDiePool>,
    pub features: Vec<ResolvedFeature>,
    pub pending_choices: Vec<PendingChoice>,
    pub carrying_capacity: i32,
    pub errors: Vec<EvalError>,
}

pub fn compute(sheet: &CharacterSheet, content: &ContentDb) -> ComputedCharacter {
    let built = build(sheet, content);
    let ctx = EvalCtx::new(&built);

    let abilities = Ability::ALL
        .iter()
        .map(|&a| AbilityView {
            ability: a,
            score: ctx.eval(&StatId::AbilityScore(a)),
            modifier: ctx.eval(&StatId::AbilityModifier(a)),
        })
        .collect();

    let saves = Ability::ALL
        .iter()
        .map(|&a| SaveView {
            ability: a,
            proficient: built.save_prof.contains(&a),
            test: ctx.d20(&StatId::SavingThrow(a)),
        })
        .collect();

    let skills = Skill::ALL
        .iter()
        .map(|&s| SkillView {
            skill: s,
            ability: s.ability(),
            proficient: built.skill_prof.contains(&s),
            expertise: built.skill_expertise.contains(&s),
            bonus: ctx.eval(&StatId::SkillBonus(s)),
            test: ctx.d20(&StatId::SkillBonus(s)),
        })
        .collect();

    let passives = [Skill::Perception, Skill::Investigation, Skill::Insight]
        .iter()
        .map(|&s| PassiveView {
            skill: s,
            value: ctx.eval(&StatId::PassiveScore(s)),
        })
        .collect();

    let mut speeds = vec![SpeedView {
        kind: MovementKind::Walk,
        value: ctx.eval(&StatId::Speed(MovementKind::Walk)),
    }];
    for k in [
        MovementKind::Fly,
        MovementKind::Swim,
        MovementKind::Climb,
        MovementKind::Burrow,
    ] {
        if built.contribs.contains_key(&StatId::Speed(k)) {
            speeds.push(SpeedView {
                kind: k,
                value: ctx.eval(&StatId::Speed(k)),
            });
        }
    }

    let resources = built
        .resources
        .iter()
        .map(|def| {
            let max = ctx.eval(&StatId::ResourceMax(def.id.clone()));
            ResourceView {
                id: def.id.clone(),
                name: def.name.clone(),
                kind: def.kind,
                current: sheet.resources.get(&def.id).copied().unwrap_or(max),
                max,
                die: def.die_at(sheet.total_level()),
                recharge: def.recharge,
            }
        })
        .collect();

    let mut spell_slots = Vec::new();
    for n in 1..=9u8 {
        let max = ctx.eval(&StatId::SpellSlotMax(SlotLevel(n)));
        if max > 0 {
            spell_slots.push(SpellSlotView { level: n, max });
        }
    }
    let spellcasting = built
        .spellcasting
        .iter()
        .map(|sp| SpellcastingView {
            source: sp.source.0.clone(),
            ability: sp.ability,
            save_dc: ctx.eval(&StatId::SpellSaveDc(sp.source.clone())),
            attack_bonus: ctx.eval(&StatId::SpellAttackBonus(sp.source.clone())),
            prepared: sp.prepared,
        })
        .collect();

    let str_mod = ctx.eval(&StatId::AbilityModifier(Ability::Str));
    let dex_mod = ctx.eval(&StatId::AbilityModifier(Ability::Dex));
    let prof = ctx.eval(&StatId::ProficiencyBonus);
    let weapons = sheet
        .weapons
        .iter()
        .map(|w| {
            let abil = match w.kind {
                WeaponKind::Melee if w.finesse => str_mod.max(dex_mod),
                WeaponKind::Melee => str_mod,
                WeaponKind::Ranged if w.finesse => str_mod.max(dex_mod),
                WeaponKind::Ranged => dex_mod,
            };
            let prof_part = if w.proficient { prof } else { 0 };
            WeaponView {
                name: w.name.clone(),
                kind: w.kind,
                attack_bonus: prof_part + abil + w.magic_bonus
                    + ctx.eval(&StatId::WeaponAttackBonus(w.kind)),
                damage: w.damage,
                damage_bonus: abil + w.magic_bonus + ctx.eval(&StatId::WeaponDamageBonus(w.kind)),
                damage_type: w.damage_type.clone(),
                mastery: w.mastery.clone(),
            }
        })
        .collect();
    let masteries_known = ctx.eval(&StatId::WeaponMasteriesKnown);
    let effects = built
        .effects
        .iter()
        .map(|(id, name)| EffectView {
            id: id.clone(),
            name: name.clone(),
            active: sheet.active_effects.iter().any(|e| e == id),
        })
        .collect();

    let level = ctx.eval(&StatId::CharacterLevel);
    let proficiency_bonus = ctx.eval(&StatId::ProficiencyBonus);
    let max_hp = ctx.explain(&StatId::MaxHitPoints);
    let armor_class = ctx.explain(&StatId::ArmorClass);
    let initiative = ctx.explain(&StatId::Initiative);
    let carrying_capacity = ctx.eval(&StatId::CarryingCapacity);
    let errors = ctx.take_errors();

    ComputedCharacter {
        name: sheet.meta.name.clone(),
        level,
        proficiency_bonus,
        max_hp,
        current_hp: sheet.hp.current,
        temp_hp: sheet.hp.temp,
        armor_class,
        initiative,
        speeds,
        abilities,
        saves,
        skills,
        passives,
        resources,
        spell_slots,
        spellcasting,
        weapons,
        masteries_known,
        effects,
        concentration: sheet.concentration.clone(),
        hit_dice: built.hit_dice.clone(),
        features: built.features.clone(),
        pending_choices: built.pending.clone(),
        carrying_capacity,
        errors,
    }
}

/// The breakdown for any single stat — powers "what is affecting this?" tooltips.
pub fn explain(sheet: &CharacterSheet, content: &ContentDb, id: &StatId) -> StatBreakdown {
    let built = build(sheet, content);
    let ctx = EvalCtx::new(&built);
    ctx.explain(id)
}

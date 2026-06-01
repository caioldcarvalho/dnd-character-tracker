//! Public entry points. [`compute`] interprets a sheet + content into a fully
//! derived [`ComputedCharacter`] (with a breakdown for the headline stats and a
//! resolved d20 test for every save/skill). [`explain`] returns the breakdown for
//! any single stat.

use crate::build::{build, PendingChoice, ResolvedFeature};
use crate::content::ContentDb;
use crate::contribution::ContribOp;
use crate::eval::{BreakdownLine, D20Test, EvalCtx, StatBreakdown};
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
    pub current: i32,
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
    /// Full provenance of the attack bonus (proficiency + ability + magic + feats).
    pub attack_breakdown: StatBreakdown,
    /// Full provenance of the damage bonus.
    pub damage_breakdown: StatBreakdown,
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

/// A non-graph breakdown line for values computed imperatively (weapon ability
/// mod, proficiency, magic bonus) so they show alongside the contribution lines.
fn manual_line(source: &str, value: i32) -> BreakdownLine {
    BreakdownLine {
        source: source.to_string(),
        op: ContribOp::Add,
        band: "add",
        value,
        note: None,
        applied: true,
    }
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
            let spent = sheet.slots_expended.get(&n).copied().unwrap_or(0) as i32;
            spell_slots.push(SpellSlotView {
                level: n,
                current: (max - spent).max(0),
                max,
            });
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
            // Which ability governs this weapon, and its label.
            let (abil, abil_name) = match w.kind {
                WeaponKind::Melee if w.finesse => {
                    if dex_mod > str_mod {
                        (dex_mod, "DEX (finesse)")
                    } else {
                        (str_mod, "STR (finesse)")
                    }
                }
                WeaponKind::Melee => (str_mod, "STR"),
                WeaponKind::Ranged if w.finesse => {
                    if dex_mod > str_mod {
                        (dex_mod, "DEX (finesse)")
                    } else {
                        (str_mod, "STR (finesse)")
                    }
                }
                WeaponKind::Ranged => (dex_mod, "DEX"),
            };
            let prof_part = if w.proficient { prof } else { 0 };
            let feat_atk = ctx.explain(&StatId::WeaponAttackBonus(w.kind));
            let feat_dmg = ctx.explain(&StatId::WeaponDamageBonus(w.kind));

            // Attack breakdown: ability + (proficiency) + (magic) + feat lines.
            let mut atk_lines = vec![manual_line(abil_name, abil)];
            if w.proficient {
                atk_lines.push(manual_line("Proficiency", prof));
            }
            if w.magic_bonus != 0 {
                atk_lines.push(manual_line("Magic weapon", w.magic_bonus));
            }
            atk_lines.extend(feat_atk.lines.iter().cloned());
            let attack_bonus = prof_part + abil + w.magic_bonus + feat_atk.total;
            let attack_breakdown = StatBreakdown {
                stat: StatId::WeaponAttackBonus(w.kind),
                label: format!("{} — Attack", w.name),
                total: attack_bonus,
                lines: atk_lines,
            };

            // Damage breakdown: ability + (magic) + feat lines (no proficiency).
            let mut dmg_lines = vec![manual_line(abil_name, abil)];
            if w.magic_bonus != 0 {
                dmg_lines.push(manual_line("Magic weapon", w.magic_bonus));
            }
            dmg_lines.extend(feat_dmg.lines.iter().cloned());
            let damage_bonus = abil + w.magic_bonus + feat_dmg.total;
            let damage_breakdown = StatBreakdown {
                stat: StatId::WeaponDamageBonus(w.kind),
                label: format!("{} — Damage", w.name),
                total: damage_bonus,
                lines: dmg_lines,
            };

            WeaponView {
                name: w.name.clone(),
                kind: w.kind,
                attack_bonus,
                damage: w.damage,
                damage_bonus,
                damage_type: w.damage_type.clone(),
                mastery: w.mastery.clone(),
                attack_breakdown,
                damage_breakdown,
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

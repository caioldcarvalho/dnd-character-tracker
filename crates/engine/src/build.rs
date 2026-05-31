//! Assembly: turn a [`CharacterSheet`] + [`ContentDb`] into the flat map of
//! contributions the evaluator reduces, plus the proficiency sets, resource
//! defs, hit-dice pools, resolved features, and the list of *pending* choices
//! that drive the level-up UI.

use crate::content::{ContentDb, Progression};
use crate::contribution::{Contribution, SourceRef};
use crate::feature::{Activation, ChoiceOptions, ChoicePoint, Feature, ProficiencyGrant};
use crate::ids::{Ability, CastingSource, MovementKind, ResourceId, Skill, SlotLevel, StatId};
use crate::resource::{HitDiePool, ResourceDef};
use crate::sheet::CharacterSheet;
use crate::value::ValueExpr;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// A contribution paired with where it came from.
#[derive(Clone, Debug)]
pub struct Sourced {
    pub source: SourceRef,
    pub contrib: Contribution,
}

pub type ContribMap = BTreeMap<StatId, Vec<Sourced>>;

/// Runtime conditions that gate `Condition`-based contributions.
#[derive(Clone, Copy, Debug, Default)]
pub struct EnvFlags {
    pub wearing_armor: bool,
    pub wielding_shield: bool,
}

/// An unresolved decision the player still has to make.
#[derive(Clone, Debug, Serialize)]
pub struct PendingChoice {
    pub key: String,
    pub prompt: String,
    pub choose: u8,
    pub options: ChoiceOptions,
    pub source: String,
}

/// A feature the character has, for display.
#[derive(Clone, Debug, Serialize)]
pub struct ResolvedFeature {
    pub name: String,
    pub description: String,
    pub source: String,
    pub kind: &'static str,
}

/// A spellcasting source the character has (one per casting class/feat).
#[derive(Clone, Debug, Serialize)]
pub struct SpellSource {
    pub source: CastingSource,
    pub ability: Ability,
    pub prepared: Option<i32>,
}

/// The fully assembled, ready-to-evaluate character.
#[derive(Debug)]
pub struct Built {
    pub contribs: ContribMap,
    pub env: EnvFlags,
    pub active: BTreeSet<String>,
    pub save_prof: BTreeSet<Ability>,
    pub skill_prof: BTreeSet<Skill>,
    pub skill_expertise: BTreeSet<Skill>,
    pub resources: Vec<ResourceDef>,
    pub hit_dice: Vec<HitDiePool>,
    pub features: Vec<ResolvedFeature>,
    pub pending: Vec<PendingChoice>,
    pub spellcasting: Vec<SpellSource>,
    /// Toggleable effects (id, name) the character has, e.g. Rage.
    pub effects: Vec<(String, String)>,
}

/// Assemble a character. Pure: no I/O.
pub fn build(sheet: &CharacterSheet, content: &ContentDb) -> Built {
    let mut b = Builder::new(sheet, content);
    b.intrinsics();
    b.species_traits();
    b.background();
    b.classes_and_subclasses();
    b.feats();
    b.spellcasting();
    b.proficiency_bonuses();
    b.exhaustion();
    b.resources_to_stats();
    b.finish()
}

/// Standard full-caster spell-slot table, indexed `[level-1][spell_level-1]`.
#[rustfmt::skip]
const FULL_CASTER_SLOTS: [[u8; 9]; 20] = [
    [2,0,0,0,0,0,0,0,0], [3,0,0,0,0,0,0,0,0], [4,2,0,0,0,0,0,0,0], [4,3,0,0,0,0,0,0,0],
    [4,3,2,0,0,0,0,0,0], [4,3,3,0,0,0,0,0,0], [4,3,3,1,0,0,0,0,0], [4,3,3,2,0,0,0,0,0],
    [4,3,3,3,1,0,0,0,0], [4,3,3,3,2,0,0,0,0], [4,3,3,3,2,1,0,0,0], [4,3,3,3,2,1,0,0,0],
    [4,3,3,3,2,1,1,0,0], [4,3,3,3,2,1,1,0,0], [4,3,3,3,2,1,1,1,0], [4,3,3,3,2,1,1,1,0],
    [4,3,3,3,2,1,1,1,1], [4,3,3,3,3,1,1,1,1], [4,3,3,3,3,2,1,1,1], [4,3,3,3,3,2,2,1,1],
];

struct Builder<'a> {
    sheet: &'a CharacterSheet,
    content: &'a ContentDb,
    contribs: ContribMap,
    save_prof: BTreeSet<Ability>,
    skill_prof: BTreeSet<Skill>,
    skill_expertise: BTreeSet<Skill>,
    resources: BTreeMap<ResourceId, ResourceDef>,
    hit_dice: BTreeMap<u8, u8>,
    features: Vec<ResolvedFeature>,
    pending: Vec<PendingChoice>,
    spellcasting: Vec<SpellSource>,
    effects: Vec<(String, String)>,
}

impl<'a> Builder<'a> {
    fn new(sheet: &'a CharacterSheet, content: &'a ContentDb) -> Self {
        Self {
            sheet,
            content,
            contribs: BTreeMap::new(),
            save_prof: BTreeSet::new(),
            skill_prof: BTreeSet::new(),
            skill_expertise: BTreeSet::new(),
            resources: BTreeMap::new(),
            hit_dice: BTreeMap::new(),
            features: Vec::new(),
            pending: Vec::new(),
            spellcasting: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn push(&mut self, source: SourceRef, contrib: Contribution) {
        self.contribs
            .entry(contrib.target.clone())
            .or_default()
            .push(Sourced { source, contrib });
    }

    fn intrinsics(&mut self) {
        let s = self.sheet;

        // Base ability scores (ASIs are added later as `Add`).
        for a in Ability::ALL {
            self.push(
                SourceRef::Base,
                Contribution::base(StatId::AbilityScore(a), ValueExpr::lit(s.base_ability(a)))
                    .note("Base score"),
            );
        }

        // Character level (sum of class levels) + per-class level.
        for entry in &s.classes {
            self.push(
                SourceRef::Class {
                    class: entry.class.clone(),
                    feature: "Levels".into(),
                    level: entry.level,
                },
                Contribution::base(StatId::ClassLevel(entry.class.clone()), ValueExpr::lit(entry.level as i32)),
            );
            self.push(
                SourceRef::Class {
                    class: entry.class.clone(),
                    feature: "Levels".into(),
                    level: entry.level,
                },
                Contribution::add(StatId::CharacterLevel, ValueExpr::lit(entry.level as i32))
                    .note(format!("{} {}", entry.class, entry.level)),
            );
            let sides = self.class_hit_die(&entry.class);
            *self.hit_dice.entry(sides).or_insert(0) += entry.level;
        }

        // Proficiency bonus by level.
        self.push(
            SourceRef::Base,
            Contribution::base(
                StatId::ProficiencyBonus,
                ValueExpr::level_table([(1, 2), (5, 3), (9, 4), (13, 5), (17, 6)]),
            )
            .note("By level"),
        );

        // Armor Class — unarmored base, then armor base (max wins), then shield.
        self.push(
            SourceRef::Base,
            Contribution::base(
                StatId::ArmorClass,
                ValueExpr::sum(vec![ValueExpr::lit(10), ValueExpr::ability_mod(Ability::Dex)]),
            )
            .note("Unarmored (10 + DEX)"),
        );
        if let Some(armor) = &s.equipment.armor {
            let dex_part = match armor.dex_cap {
                Some(cap) => ValueExpr::min(vec![ValueExpr::ability_mod(Ability::Dex), ValueExpr::lit(cap)]),
                None => ValueExpr::ability_mod(Ability::Dex),
            };
            self.push(
                SourceRef::Item { id: armor.name.clone() },
                Contribution::base(
                    StatId::ArmorClass,
                    ValueExpr::sum(vec![ValueExpr::lit(armor.base_ac), dex_part]),
                )
                .note(armor.name.clone()),
            );
        }
        if s.equipment.shield {
            self.push(
                SourceRef::Item { id: "Shield".into() },
                Contribution::add(StatId::ArmorClass, ValueExpr::lit(2)).note("Shield"),
            );
        }

        // Initiative, saves, skills, passives.
        self.push(
            SourceRef::Base,
            Contribution::add(StatId::Initiative, ValueExpr::ability_mod(Ability::Dex)).note("DEX"),
        );
        self.push(
            SourceRef::Base,
            Contribution::base(StatId::AttacksPerAction, ValueExpr::lit(1)).note("One attack"),
        );
        for a in Ability::ALL {
            self.push(
                SourceRef::Base,
                Contribution::add(StatId::SavingThrow(a), ValueExpr::ability_mod(a)).note(a.abbr()),
            );
        }
        for sk in Skill::ALL {
            self.push(
                SourceRef::Base,
                Contribution::add(StatId::SkillBonus(sk), ValueExpr::ability_mod(sk.ability()))
                    .note(sk.ability().abbr()),
            );
            self.push(
                SourceRef::Base,
                Contribution::base(StatId::PassiveScore(sk), ValueExpr::lit(10)).note("Base 10"),
            );
            self.push(
                SourceRef::Base,
                Contribution::add(StatId::PassiveScore(sk), ValueExpr::stat(StatId::SkillBonus(sk)))
                    .note(sk.label()),
            );
        }

        // Walk speed (species base, else 30).
        let (speed, speed_src) = match s
            .species
            .as_deref()
            .and_then(|id| self.content.species_def(id))
        {
            Some(sp) => (sp.speed, SourceRef::Species { id: sp.id.clone() }),
            None => (30, SourceRef::Base),
        };
        self.push(
            speed_src,
            Contribution::base(StatId::Speed(MovementKind::Walk), ValueExpr::lit(speed)).note("Base speed"),
        );

        // Carrying capacity.
        self.push(
            SourceRef::Base,
            Contribution::base(
                StatId::CarryingCapacity,
                ValueExpr::scaled(ValueExpr::stat(StatId::AbilityScore(Ability::Str)), ValueExpr::lit(15)),
            )
            .note("STR × 15"),
        );

        // Max HP.
        self.hit_points();
    }

    fn hit_points(&mut self) {
        let s = self.sheet;
        let mut first = true;
        for entry in &s.classes {
            let sides = self.class_hit_die(&entry.class);
            let avg = sides as i32 / 2 + 1;
            let src = SourceRef::Class {
                class: entry.class.clone(),
                feature: "Hit Points".into(),
                level: entry.level,
            };
            if first {
                self.push(
                    src.clone(),
                    Contribution::add(StatId::MaxHitPoints, ValueExpr::lit(sides as i32))
                        .note(format!("L1 max d{sides}")),
                );
                if entry.level > 1 {
                    self.push(
                        src,
                        Contribution::add(StatId::MaxHitPoints, ValueExpr::lit(avg * (entry.level as i32 - 1)))
                            .note(format!("avg d{sides} ×{}", entry.level - 1)),
                    );
                }
                first = false;
            } else {
                self.push(
                    src,
                    Contribution::add(StatId::MaxHitPoints, ValueExpr::lit(avg * entry.level as i32))
                        .note(format!("avg d{sides} ×{}", entry.level)),
                );
            }
        }
        // CON modifier per character level.
        let total = s.total_level() as i32;
        if total > 0 {
            self.push(
                SourceRef::Base,
                Contribution::add(
                    StatId::MaxHitPoints,
                    ValueExpr::scaled(ValueExpr::ability_mod(Ability::Con), ValueExpr::lit(total)),
                )
                .note(format!("CON ×{total}")),
            );
        }
    }

    fn species_traits(&mut self) {
        let Some(sp) = self.sheet.species.as_deref().and_then(|id| self.content.species_def(id)) else {
            return;
        };
        let traits = sp.traits.clone();
        let id = sp.id.clone();
        for t in &traits {
            self.apply_feature(SourceRef::Species { id: id.clone() }, t, "Species Trait");
        }
    }

    /// 2024 background: ability-score spread (a choice), skill/tool proficiencies,
    /// and the origin feat.
    fn background(&mut self) {
        let Some(bg) = self
            .sheet
            .background
            .as_deref()
            .and_then(|id| self.content.background(id))
            .cloned()
        else {
            return;
        };
        let src = SourceRef::Background { id: bg.id.clone() };

        for sk in &bg.skills {
            self.skill_prof.insert(*sk);
        }
        // Tool proficiencies don't affect computed stats yet (recorded for display).
        self.features.push(ResolvedFeature {
            name: bg.name.clone(),
            description: bg.description.clone(),
            source: src.label(),
            kind: "Background",
        });

        // The +2/+1 or +1/+1/+1 ability spread is a player choice.
        if !bg.abilities.is_empty() {
            let choice = ChoicePoint {
                id: "background-abilities".into(),
                prompt: "Increase your ability scores (+2 and +1, or +1/+1/+1)".into(),
                choose: 3,
                options: ChoiceOptions::AbilityScores {
                    from: bg.abilities.clone(),
                    points: 3,
                    max_each: 2,
                },
            };
            self.handle_choice(src.clone(), &choice);
        }

        if let Some(feat_id) = &bg.origin_feat {
            if let Some(f) = self.content.feat(feat_id).cloned() {
                self.apply_feature(SourceRef::Feat { id: feat_id.clone() }, &f, "Origin Feat");
            }
        }
    }

    fn classes_and_subclasses(&mut self) {
        let classes = self.sheet.classes.clone();
        let mut is_first = true;
        for entry in &classes {
            let Some(class_def) = self.content.class(&entry.class).cloned() else {
                continue;
            };
            // Saving throws — only the first class grants them.
            if is_first {
                for a in &class_def.saving_throws {
                    self.save_prof.insert(*a);
                }
            }
            is_first = false;

            // Class features up to the character's level in this class.
            for (lvl, grant) in class_def.levels.range(1..=entry.level) {
                for f in &grant.features {
                    let src = SourceRef::Class {
                        class: entry.class.clone(),
                        feature: f.name.clone(),
                        level: *lvl,
                    };
                    self.apply_feature(src, f, "Class Feature");
                }
                for ch in &grant.choices {
                    let src = SourceRef::Class {
                        class: entry.class.clone(),
                        feature: format!("Level {lvl}"),
                        level: *lvl,
                    };
                    self.handle_choice(src, ch);
                }
            }

            // Subclass.
            match entry.subclass.as_deref() {
                Some(sub_id) => {
                    if let Some(sub) = self.content.subclass(sub_id).cloned() {
                        for (lvl, grant) in sub.levels.range(1..=entry.level) {
                            for f in &grant.features {
                                let src = SourceRef::Subclass {
                                    subclass: sub.name.clone(),
                                    feature: f.name.clone(),
                                    level: *lvl,
                                };
                                self.apply_feature(src, f, "Subclass Feature");
                            }
                            for ch in &grant.choices {
                                let src = SourceRef::Subclass {
                                    subclass: sub.name.clone(),
                                    feature: format!("Level {lvl}"),
                                    level: *lvl,
                                };
                                self.handle_choice(src, ch);
                            }
                        }
                    }
                }
                None if entry.level >= class_def.subclass_level => {
                    self.pending.push(PendingChoice {
                        key: format!("{}-subclass", entry.class),
                        prompt: format!("Choose a {} subclass", class_def.name),
                        choose: 1,
                        options: ChoiceOptions::Subclass,
                        source: class_def.name.clone(),
                    });
                }
                None => {}
            }
        }
    }

    fn feats(&mut self) {
        let feats = self.sheet.feats.clone();
        for id in &feats {
            if let Some(f) = self.content.feat(id).cloned() {
                self.apply_feature(SourceRef::Feat { id: id.clone() }, &f, "Feat");
            }
        }
    }

    /// Spellcasting: per-source save DC & attack bonus (traceable stats), plus the
    /// shared spell-slot pool from the multiclass caster level.
    fn spellcasting(&mut self) {
        let classes = self.sheet.classes.clone();
        let (mut full, mut half, mut third) = (0u32, 0u32, 0u32);

        for entry in &classes {
            let Some(class_def) = self.content.class(&entry.class).cloned() else {
                continue;
            };
            // A class's spec, or a subclass that grants casting (e.g. an Eldritch Knight).
            let spec = class_def.spellcasting.clone().or_else(|| {
                entry
                    .subclass
                    .as_deref()
                    .and_then(|s| self.content.subclass(s))
                    .and_then(|s| s.spellcasting.clone())
            });
            let Some(spec) = spec else { continue };

            match spec.progression {
                Progression::Full => full += entry.level as u32,
                Progression::Half => half += entry.level as u32,
                Progression::Third => third += entry.level as u32,
                Progression::Pact | Progression::None => {}
            }

            let cs = CastingSource::new(entry.class.0.clone());
            let src = SourceRef::Class {
                class: entry.class.clone(),
                feature: "Spellcasting".into(),
                level: entry.level,
            };
            self.push(
                src.clone(),
                Contribution::base(StatId::SpellSaveDc(cs.clone()), ValueExpr::lit(8)).note("Base 8"),
            );
            self.push(
                src.clone(),
                Contribution::add(StatId::SpellSaveDc(cs.clone()), ValueExpr::stat(StatId::ProficiencyBonus))
                    .note("Proficiency"),
            );
            self.push(
                src.clone(),
                Contribution::add(StatId::SpellSaveDc(cs.clone()), ValueExpr::ability_mod(spec.ability))
                    .note(spec.ability.abbr()),
            );
            self.push(
                src.clone(),
                Contribution::add(StatId::SpellAttackBonus(cs.clone()), ValueExpr::stat(StatId::ProficiencyBonus))
                    .note("Proficiency"),
            );
            self.push(
                src,
                Contribution::add(StatId::SpellAttackBonus(cs.clone()), ValueExpr::ability_mod(spec.ability))
                    .note(spec.ability.abbr()),
            );

            let prepared = spec
                .prepared_per_level
                .as_ref()
                .and_then(|t| t.range(..=entry.level).next_back().map(|(_, &v)| v));
            self.spellcasting.push(SpellSource {
                source: cs,
                ability: spec.ability,
                prepared,
            });
        }

        // Shared spell slots from the combined caster level.
        let caster_level = (full + half / 2 + third / 3).min(20) as usize;
        if caster_level > 0 {
            let slots = FULL_CASTER_SLOTS[caster_level - 1];
            for (i, &count) in slots.iter().enumerate() {
                if count > 0 {
                    self.push(
                        SourceRef::Manual { label: "Spellcasting".into() },
                        Contribution::base(
                            StatId::SpellSlotMax(SlotLevel((i + 1) as u8)),
                            ValueExpr::lit(count as i32),
                        )
                        .note("Caster level"),
                    );
                }
            }
        }
    }

    fn apply_feature(&mut self, source: SourceRef, feat: &Feature, kind: &'static str) {
        for c in &feat.contributions {
            self.push(source.clone(), c.clone());
        }
        for p in &feat.proficiencies {
            self.apply_prof(p);
        }
        for r in &feat.resources {
            self.resources.insert(r.id.clone(), r.clone());
        }
        for ch in &feat.choices {
            self.handle_choice(source.clone(), ch);
        }
        if matches!(feat.activation, Activation::Toggle { .. }) {
            self.effects.push((feat.id.clone(), feat.name.clone()));
        }
        self.features.push(ResolvedFeature {
            name: feat.name.clone(),
            description: feat.description.clone(),
            source: source.label(),
            kind,
        });
    }

    fn apply_prof(&mut self, p: &ProficiencyGrant) {
        match p {
            ProficiencyGrant::Skill { skill, expertise } => {
                self.skill_prof.insert(*skill);
                if *expertise {
                    self.skill_expertise.insert(*skill);
                }
            }
            ProficiencyGrant::SavingThrow { ability } => {
                self.save_prof.insert(*ability);
            }
            // Tool/weapon/armor/language proficiencies don't affect computed stats yet.
            _ => {}
        }
    }

    /// Apply whatever the player has picked for a choice, and flag it pending if
    /// not yet fully resolved.
    fn handle_choice(&mut self, source: SourceRef, ch: &ChoicePoint) {
        let picks: Vec<String> = self.sheet.picks_for(&ch.id).to_vec();
        if picks.len() < ch.choose as usize {
            self.pending.push(PendingChoice {
                key: ch.id.clone(),
                prompt: ch.prompt.clone(),
                choose: ch.choose,
                options: ch.options.clone(),
                source: source.label(),
            });
        }
        for pick in &picks {
            match &ch.options {
                ChoiceOptions::Skills { .. } => {
                    if let Some(sk) = parse_skill(pick) {
                        self.skill_prof.insert(sk);
                    }
                }
                ChoiceOptions::Expertise { .. } => {
                    if let Some(sk) = parse_skill(pick) {
                        self.skill_prof.insert(sk);
                        self.skill_expertise.insert(sk);
                    }
                }
                ChoiceOptions::AbilityScoreIncrease => {
                    if let Some(ab) = parse_ability(pick) {
                        self.push(
                            source.clone(),
                            Contribution::add(StatId::AbilityScore(ab), ValueExpr::lit(1)).note("ASI"),
                        );
                    } else if let Some(f) = self.content.feat(pick).cloned() {
                        self.apply_feature(SourceRef::Feat { id: pick.clone() }, &f, "Feat");
                    }
                }
                ChoiceOptions::AbilityScores { .. } => {
                    // Each recorded pick is one ability gaining +1 (pick the same
                    // ability twice for +2). The UI enforces points / max_each.
                    if let Some(ab) = parse_ability(pick) {
                        self.push(
                            source.clone(),
                            Contribution::add(StatId::AbilityScore(ab), ValueExpr::lit(1))
                                .note("Background"),
                        );
                    }
                }
                ChoiceOptions::Feat { .. } => {
                    if let Some(f) = self.content.feat(pick).cloned() {
                        self.apply_feature(SourceRef::Feat { id: pick.clone() }, &f, "Feat");
                    }
                }
                ChoiceOptions::Named { from } => {
                    if let Some(opt) = from.iter().find(|o| &o.id == pick) {
                        for c in &opt.contributions {
                            self.push(source.clone(), c.clone());
                        }
                        for p in &opt.proficiencies {
                            self.apply_prof(p);
                        }
                    }
                }
                // Subclass picks live on the ClassEntry; spells/mastery have no stat effect yet.
                ChoiceOptions::Subclass
                | ChoiceOptions::Spells { .. }
                | ChoiceOptions::WeaponMastery { .. } => {}
            }
        }
    }

    fn proficiency_bonuses(&mut self) {
        for a in self.save_prof.clone() {
            self.push(
                SourceRef::Manual { label: "Proficiency".into() },
                Contribution::add(StatId::SavingThrow(a), ValueExpr::stat(StatId::ProficiencyBonus))
                    .note("Proficient"),
            );
        }
        for sk in self.skill_prof.clone() {
            self.push(
                SourceRef::Manual { label: "Proficiency".into() },
                Contribution::add(StatId::SkillBonus(sk), ValueExpr::stat(StatId::ProficiencyBonus))
                    .note("Proficient"),
            );
        }
        for sk in self.skill_expertise.clone() {
            self.push(
                SourceRef::Manual { label: "Expertise".into() },
                Contribution::add(StatId::SkillBonus(sk), ValueExpr::stat(StatId::ProficiencyBonus))
                    .note("Expertise"),
            );
        }
    }

    fn exhaustion(&mut self) {
        let e = self.sheet.exhaustion as i32;
        if e == 0 {
            return;
        }
        let src = SourceRef::Condition { id: "exhaustion".into() };
        let note = format!("Exhaustion {e}");
        for a in Ability::ALL {
            self.push(
                src.clone(),
                Contribution::add(StatId::SavingThrow(a), ValueExpr::lit(-2 * e)).note(note.clone()),
            );
        }
        for sk in Skill::ALL {
            self.push(
                src.clone(),
                Contribution::add(StatId::SkillBonus(sk), ValueExpr::lit(-2 * e)).note(note.clone()),
            );
        }
        self.push(
            src,
            Contribution::add(StatId::Speed(MovementKind::Walk), ValueExpr::lit(-5 * e)).note(note),
        );
    }

    fn resources_to_stats(&mut self) {
        let defs: Vec<ResourceDef> = self.resources.values().cloned().collect();
        for def in defs {
            self.push(
                SourceRef::Manual { label: def.name.clone() },
                Contribution::base(StatId::ResourceMax(def.id.clone()), def.max.clone()).note(def.name.clone()),
            );
        }
    }

    fn class_hit_die(&self, class: &crate::ids::ClassId) -> u8 {
        self.content.class(class).map(|c| c.hit_die).unwrap_or(8)
    }

    fn finish(self) -> Built {
        let hit_dice = self
            .hit_dice
            .into_iter()
            .map(|(sides, total)| HitDiePool {
                sides,
                total,
                spent: self.sheet.hit_dice_spent.get(&sides).copied().unwrap_or(0),
            })
            .collect();
        let env = EnvFlags {
            wearing_armor: self.sheet.equipment.armor.is_some(),
            wielding_shield: self.sheet.equipment.shield,
        };
        Built {
            contribs: self.contribs,
            env,
            active: self.sheet.active_effects.iter().cloned().collect(),
            save_prof: self.save_prof,
            skill_prof: self.skill_prof,
            skill_expertise: self.skill_expertise,
            resources: self.resources.into_values().collect(),
            hit_dice,
            features: self.features,
            pending: self.pending,
            spellcasting: self.spellcasting,
            effects: self.effects,
        }
    }
}

fn parse_skill(s: &str) -> Option<Skill> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
}
fn parse_ability(s: &str) -> Option<Ability> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
}

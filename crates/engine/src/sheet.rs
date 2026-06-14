//! [`CharacterSheet`] — the persisted file. It holds *build choices* (species,
//! background, classes, ability scores, feats, recorded level-up decisions) and
//! *runtime state* (HP, expended resources, conditions, active effects). The
//! engine derives everything else.

use crate::ids::{Ability, ClassId, ResourceId, WeaponKind};
use crate::value::Dice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Meta {
    pub name: String,
    #[serde(default)]
    pub player: String,
    #[serde(default)]
    pub id: String,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassEntry {
    pub class: ClassId,
    pub level: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subclass: Option<String>,
}

/// A resolved [`ChoicePoint`](crate::feature::ChoicePoint): `key` matches the
/// choice's id, `picks` are the selected option ids / values.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedChoice {
    pub key: String,
    #[serde(default)]
    pub picks: Vec<String>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArmorKind {
    Light,
    Medium,
    Heavy,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmorItem {
    pub name: String,
    pub base_ac: i32,
    pub kind: ArmorKind,
    /// Max DEX bonus allowed: `None` = light (full DEX), `Some(2)` = medium,
    /// `Some(0)` = heavy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dex_cap: Option<i32>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Equipment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor: Option<ArmorItem>,
    #[serde(default)]
    pub shield: bool,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HpState {
    #[serde(default)]
    pub current: i32,
    #[serde(default)]
    pub temp: i32,
    /// Optional per-level HP rolls; when empty, the engine uses fixed averages.
    #[serde(default)]
    pub rolled: Vec<u8>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponInstance {
    pub name: String,
    #[serde(default)]
    pub kind: WeaponKind,
    pub damage: Dice,
    #[serde(default)]
    pub damage_type: String,
    /// Finesse weapons may use DEX instead of STR.
    #[serde(default)]
    pub finesse: bool,
    #[serde(default)]
    pub two_handed: bool,
    /// Enhancement bonus from a magic weapon (applies to attack and damage).
    #[serde(default)]
    pub magic_bonus: i32,
    #[serde(default = "yes")]
    pub proficient: bool,
    /// 2024 Weapon Mastery property (Vex, Topple, …) the wielder may use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mastery: Option<String>,
}

fn yes() -> bool {
    true
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeathSaves {
    #[serde(default)]
    pub successes: u8,
    #[serde(default)]
    pub failures: u8,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSheet {
    #[serde(default)]
    pub meta: Meta,
    /// Base ability scores, before any modifiers from species/background/feats.
    pub abilities: BTreeMap<Ability, i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub species: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default)]
    pub classes: Vec<ClassEntry>,
    #[serde(default)]
    pub feats: Vec<String>,
    #[serde(default)]
    pub choices: Vec<RecordedChoice>,
    #[serde(default)]
    pub hp: HpState,
    /// Current value of each resource pool (max is computed).
    #[serde(default)]
    pub resources: BTreeMap<ResourceId, i32>,
    /// Spell slots expended, by slot level (max is computed).
    #[serde(default)]
    pub slots_expended: BTreeMap<u8, u8>,
    /// Hit dice spent, by die size.
    #[serde(default)]
    pub hit_dice_spent: BTreeMap<u8, u8>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub exhaustion: u8,
    /// Toggles / active effects currently on (matched by `Condition::EffectActive`).
    #[serde(default)]
    pub active_effects: Vec<String>,
    #[serde(default)]
    pub equipment: Equipment,
    #[serde(default)]
    pub weapons: Vec<WeaponInstance>,
    /// Spell ids the character knows (for Known-caster classes like Sorcerer/Bard).
    #[serde(default)]
    pub known_spells: Vec<String>,
    /// Spell ids the character has prepared (for Prepared-caster classes like Cleric/Wizard).
    #[serde(default)]
    pub prepared_spells: Vec<String>,
    /// The spell currently being concentrated on, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concentration: Option<String>,
    #[serde(default)]
    pub death_saves: DeathSaves,
    #[serde(default)]
    pub inspiration: bool,
    /// Free-form player notes (markdown/plain text).
    #[serde(default)]
    pub notes: String,
}

impl CharacterSheet {
    /// A blank sheet with all abilities at 10.
    pub fn new(name: impl Into<String>) -> Self {
        let abilities = Ability::ALL.iter().map(|&a| (a, 10)).collect();
        Self {
            meta: Meta {
                name: name.into(),
                ..Default::default()
            },
            abilities,
            species: None,
            background: None,
            classes: Vec::new(),
            feats: Vec::new(),
            choices: Vec::new(),
            hp: HpState::default(),
            resources: BTreeMap::new(),
            slots_expended: BTreeMap::new(),
            hit_dice_spent: BTreeMap::new(),
            conditions: Vec::new(),
            exhaustion: 0,
            active_effects: Vec::new(),
            equipment: Equipment::default(),
            weapons: Vec::new(),
            known_spells: Vec::new(),
            prepared_spells: Vec::new(),
            concentration: None,
            death_saves: DeathSaves::default(),
            inspiration: false,
            notes: String::new(),
        }
    }

    pub fn with_ability(mut self, a: Ability, score: i32) -> Self {
        self.abilities.insert(a, score);
        self
    }
    pub fn with_background(mut self, id: impl Into<String>) -> Self {
        self.background = Some(id.into());
        self
    }
    pub fn with_species(mut self, id: impl Into<String>) -> Self {
        self.species = Some(id.into());
        self
    }
    pub fn with_class(mut self, class: impl Into<ClassId>, level: u8, subclass: Option<&str>) -> Self {
        self.classes.push(ClassEntry {
            class: class.into(),
            level,
            subclass: subclass.map(|s| s.to_string()),
        });
        self
    }
    pub fn with_choice(mut self, key: impl Into<String>, picks: &[&str]) -> Self {
        self.choices.push(RecordedChoice {
            key: key.into(),
            picks: picks.iter().map(|s| s.to_string()).collect(),
        });
        self
    }
    pub fn with_feat(mut self, feat: impl Into<String>) -> Self {
        self.feats.push(feat.into());
        self
    }

    pub fn base_ability(&self, a: Ability) -> i32 {
        self.abilities.get(&a).copied().unwrap_or(10)
    }
    pub fn total_level(&self) -> u8 {
        self.classes.iter().map(|c| c.level).sum()
    }
    pub fn class_level(&self, class: &ClassId) -> u8 {
        self.classes
            .iter()
            .filter(|c| &c.class == class)
            .map(|c| c.level)
            .sum()
    }
    pub fn picks_for(&self, key: &str) -> &[String] {
        self.choices
            .iter()
            .find(|c| c.key == key)
            .map(|c| c.picks.as_slice())
            .unwrap_or(&[])
    }
}

//! [`Feature`] — the one primitive that models class features, feats, species
//! traits, item properties, spell effects, and conditions. Each is a bundle of
//! contributions + proficiency grants + resources + spell grants + required
//! choices, applied from a different [`SourceRef`](crate::contribution::SourceRef).

use crate::contribution::Contribution;
use crate::ids::{Ability, Skill};
use crate::resource::ResourceDef;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Activation {
    /// Always on.
    Passive,
    /// On/off; contributes only while on (gated via `Condition::EffectActive`).
    Toggle {
        #[serde(default)]
        default_on: bool,
    },
    /// Used as an action/bonus action, usually spending a resource.
    Activated,
    Reaction,
}

impl Default for Activation {
    fn default() -> Self {
        Activation::Passive
    }
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ProficiencyGrant {
    Skill {
        skill: Skill,
        #[serde(default)]
        expertise: bool,
    },
    SavingThrow {
        ability: Ability,
    },
    Tool {
        tool: String,
    },
    Weapon {
        weapon: String,
    },
    Armor {
        armor: String,
    },
    Language {
        language: String,
    },
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpellGrant {
    pub spell: String,
    #[serde(default)]
    pub always_prepared: bool,
}

/// A decision the player must make (skill picks, fighting style, ASI, subclass…).
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChoicePoint {
    /// Globally unique key; a [`RecordedChoice`](crate::sheet::RecordedChoice)
    /// with the same key resolves it.
    pub id: String,
    pub prompt: String,
    #[serde(default = "one")]
    pub choose: u8,
    pub options: ChoiceOptions,
}

fn one() -> u8 {
    1
}
fn two() -> u8 {
    2
}
fn three() -> u8 {
    3
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ChoiceOptions {
    Skills {
        from: Vec<Skill>,
    },
    Expertise {
        from: Vec<Skill>,
    },
    /// Pick a feat, optionally restricted to a category (e.g. `"fighting-style"`).
    Feat {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        category: Option<String>,
    },
    /// The level-up ASI: +2 to one ability or +1 to two (among all six), or a feat.
    AbilityScoreIncrease,
    /// A restricted spread of ability-score points (2024 background: distribute
    /// `points` total among `from`, at most `max_each` to any one ability).
    AbilityScores {
        from: Vec<Ability>,
        #[serde(default = "three")]
        points: u8,
        #[serde(default = "two")]
        max_each: u8,
    },
    Subclass,
    Spells {
        #[serde(default)]
        from: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_level: Option<u8>,
    },
    WeaponMastery {
        #[serde(default)]
        from: Vec<String>,
    },
    /// Arbitrary named options, each granting some effects when picked.
    Named {
        from: Vec<NamedOption>,
    },
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NamedOption {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub contributions: Vec<Contribution>,
    #[serde(default)]
    pub proficiencies: Vec<ProficiencyGrant>,
}

/// The universal effect bundle.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Optional tag for feat filtering in choices, e.g. `"origin"`,
    /// `"fighting-style"`, `"epic-boon"`. Ignored by the math.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub activation: Activation,
    #[serde(default)]
    pub contributions: Vec<Contribution>,
    #[serde(default)]
    pub proficiencies: Vec<ProficiencyGrant>,
    #[serde(default)]
    pub resources: Vec<ResourceDef>,
    #[serde(default)]
    pub spell_grants: Vec<SpellGrant>,
    #[serde(default)]
    pub choices: Vec<ChoicePoint>,
}

impl Feature {
    /// Minimal feature with just an id + name (builder-friendly for tests).
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            category: None,
            activation: Activation::Passive,
            contributions: Vec::new(),
            proficiencies: Vec::new(),
            resources: Vec::new(),
            spell_grants: Vec::new(),
            choices: Vec::new(),
        }
    }
    pub fn with_contribution(mut self, c: Contribution) -> Self {
        self.contributions.push(c);
        self
    }
    pub fn with_resource(mut self, r: ResourceDef) -> Self {
        self.resources.push(r);
        self
    }
    pub fn with_proficiency(mut self, p: ProficiencyGrant) -> Self {
        self.proficiencies.push(p);
        self
    }
    pub fn with_choice(mut self, c: ChoicePoint) -> Self {
        self.choices.push(c);
        self
    }
}

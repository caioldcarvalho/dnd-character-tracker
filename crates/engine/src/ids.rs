//! Identifiers. [`StatId`] is the (mostly) closed set of computable nodes; the
//! variants that carry data are parameterized by small enums or — where the id is
//! inherently content-driven — by string newtypes (`ClassId`, `ResourceId`, …).

use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ability {
    Str,
    Dex,
    Con,
    Int,
    Wis,
    Cha,
}

impl Ability {
    pub const ALL: [Ability; 6] = [
        Ability::Str,
        Ability::Dex,
        Ability::Con,
        Ability::Int,
        Ability::Wis,
        Ability::Cha,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Ability::Str => "Strength",
            Ability::Dex => "Dexterity",
            Ability::Con => "Constitution",
            Ability::Int => "Intelligence",
            Ability::Wis => "Wisdom",
            Ability::Cha => "Charisma",
        }
    }

    pub fn abbr(self) -> &'static str {
        match self {
            Ability::Str => "STR",
            Ability::Dex => "DEX",
            Ability::Con => "CON",
            Ability::Int => "INT",
            Ability::Wis => "WIS",
            Ability::Cha => "CHA",
        }
    }
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Skill {
    Acrobatics,
    AnimalHandling,
    Arcana,
    Athletics,
    Deception,
    History,
    Insight,
    Intimidation,
    Investigation,
    Medicine,
    Nature,
    Perception,
    Performance,
    Persuasion,
    Religion,
    SleightOfHand,
    Stealth,
    Survival,
}

impl Skill {
    pub const ALL: [Skill; 18] = [
        Skill::Acrobatics,
        Skill::AnimalHandling,
        Skill::Arcana,
        Skill::Athletics,
        Skill::Deception,
        Skill::History,
        Skill::Insight,
        Skill::Intimidation,
        Skill::Investigation,
        Skill::Medicine,
        Skill::Nature,
        Skill::Perception,
        Skill::Performance,
        Skill::Persuasion,
        Skill::Religion,
        Skill::SleightOfHand,
        Skill::Stealth,
        Skill::Survival,
    ];

    /// The default governing ability for this skill.
    pub fn ability(self) -> Ability {
        use Ability::*;
        use Skill::*;
        match self {
            Athletics => Str,
            Acrobatics | SleightOfHand | Stealth => Dex,
            Arcana | History | Investigation | Nature | Religion => Int,
            AnimalHandling | Insight | Medicine | Perception | Survival => Wis,
            Deception | Intimidation | Performance | Persuasion => Cha,
        }
    }

    pub fn label(self) -> &'static str {
        use Skill::*;
        match self {
            Acrobatics => "Acrobatics",
            AnimalHandling => "Animal Handling",
            Arcana => "Arcana",
            Athletics => "Athletics",
            Deception => "Deception",
            History => "History",
            Insight => "Insight",
            Intimidation => "Intimidation",
            Investigation => "Investigation",
            Medicine => "Medicine",
            Nature => "Nature",
            Perception => "Perception",
            Performance => "Performance",
            Persuasion => "Persuasion",
            Religion => "Religion",
            SleightOfHand => "Sleight of Hand",
            Stealth => "Stealth",
            Survival => "Survival",
        }
    }
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MovementKind {
    Walk,
    Fly,
    Swim,
    Climb,
    Burrow,
}

impl MovementKind {
    pub fn label(self) -> &'static str {
        match self {
            MovementKind::Walk => "Walk",
            MovementKind::Fly => "Fly",
            MovementKind::Swim => "Swim",
            MovementKind::Climb => "Climb",
            MovementKind::Burrow => "Burrow",
        }
    }
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WeaponKind {
    Melee,
    Ranged,
}

impl Default for WeaponKind {
    fn default() -> Self {
        WeaponKind::Melee
    }
}

impl WeaponKind {
    pub fn label(self) -> &'static str {
        match self {
            WeaponKind::Melee => "Melee",
            WeaponKind::Ranged => "Ranged",
        }
    }
}

macro_rules! string_newtype {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(s: impl Into<String>) -> Self {
                Self(s.into())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }
        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_newtype!(
    /// A class id, e.g. `"fighter"`. Content-driven.
    #[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
    ClassId
);
string_newtype!(
    /// A resource pool id, e.g. `"superiority-dice"`. Content-driven.
    #[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
    ResourceId
);
string_newtype!(
    /// A spellcasting source id (usually a class id), e.g. `"wizard"`.
    #[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
    CastingSource
);

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SlotLevel(pub u8);

/// Every computable value in the character. The evaluator must handle each
/// variant, so adding a stat is a compiler-guided checklist.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "of", rename_all = "kebab-case")]
pub enum StatId {
    CharacterLevel,
    ClassLevel(ClassId),
    AbilityScore(Ability),
    AbilityModifier(Ability),
    ProficiencyBonus,
    MaxHitPoints,
    ArmorClass,
    Initiative,
    Speed(MovementKind),
    SavingThrow(Ability),
    SkillBonus(Skill),
    PassiveScore(Skill),
    SpellSaveDc(CastingSource),
    SpellAttackBonus(CastingSource),
    ResourceMax(ResourceId),
    SpellSlotMax(SlotLevel),
    AttacksPerAction,
    WeaponAttackBonus(WeaponKind),
    WeaponDamageBonus(WeaponKind),
    WeaponMasteriesKnown,
    CarryingCapacity,
    /// Escape hatch for homebrew / not-yet-modeled stats.
    Custom(String),
}

impl StatId {
    /// True for stats resolved as a d20 test (so adv/disadv & auto-outcome apply).
    pub fn is_d20_test(&self) -> bool {
        matches!(self, StatId::SavingThrow(_) | StatId::SkillBonus(_))
    }

    /// Human-readable label for the breakdown UI.
    pub fn label(&self) -> String {
        match self {
            StatId::CharacterLevel => "Character Level".into(),
            StatId::ClassLevel(c) => format!("{} Level", c.0),
            StatId::AbilityScore(a) => format!("{} Score", a.label()),
            StatId::AbilityModifier(a) => format!("{} Modifier", a.label()),
            StatId::ProficiencyBonus => "Proficiency Bonus".into(),
            StatId::MaxHitPoints => "Max Hit Points".into(),
            StatId::ArmorClass => "Armor Class".into(),
            StatId::Initiative => "Initiative".into(),
            StatId::Speed(m) => format!("{} Speed", m.label()),
            StatId::SavingThrow(a) => format!("{} Save", a.abbr()),
            StatId::SkillBonus(s) => s.label().to_string(),
            StatId::PassiveScore(s) => format!("Passive {}", s.label()),
            StatId::SpellSaveDc(c) => format!("Spell Save DC ({})", c.0),
            StatId::SpellAttackBonus(c) => format!("Spell Attack ({})", c.0),
            StatId::ResourceMax(r) => format!("{} (max)", r.0),
            StatId::SpellSlotMax(l) => format!("Level {} Slots", l.0),
            StatId::AttacksPerAction => "Attacks per Action".into(),
            StatId::WeaponAttackBonus(k) => format!("{} Attack Bonus", k.label()),
            StatId::WeaponDamageBonus(k) => format!("{} Damage Bonus", k.label()),
            StatId::WeaponMasteriesKnown => "Weapon Masteries Known".into(),
            StatId::CarryingCapacity => "Carrying Capacity".into(),
            StatId::Custom(s) => s.clone(),
        }
    }
}

//! Spell definitions — the rulebook for spells as data.
//!
//! [`SpellDef`] is authored in JSON under `content/spells/` and embedded into
//! the binary at build time (see `build.rs`). The engine loads them into
//! [`ContentDb`](crate::content::ContentDb) and exposes them via
//! [`Catalog`](crate::catalog::Catalog).

use serde::{Deserialize, Serialize};

use crate::ids::Ability;

/// Whether a spell's attack is melee or ranged.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpellAttack {
    Melee,
    Ranged,
}

/// The saving throw a spell forces and the effect on a success.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellSave {
    /// The ability the TARGET uses for its saving throw.
    pub ability: Ability,
    /// What happens on a successful save, e.g. `"half damage on a success"`.
    pub effect: String,
}

/// Damage a spell deals on a hit or failed save.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellDamage {
    /// Dice expression, e.g. `"1d10"`, `"3d6"`, `"3 × 1d4+1"`.
    pub dice: String,
    /// Damage type, e.g. `"fire"`, `"cold"`, `"healing"`.
    pub damage_type: String,
}

/// The eight schools of magic in D&D 2024.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpellSchool {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromancy,
    Transmutation,
}

/// The material, verbal, and somatic components of a spell.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellComponents {
    #[serde(default)]
    pub verbal: bool,
    #[serde(default)]
    pub somatic: bool,
    /// Specific material component required (omit if none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
}

/// A single spell definition, as authored in `content/spells/*.json`.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpellDef {
    /// Kebab-case unique identifier, e.g. `"fire-bolt"`.
    pub id: String,
    pub name: String,
    /// Spell level; 0 = cantrip.
    pub level: u8,
    pub school: SpellSchool,
    /// Class ids that have this spell on their spell list, e.g. `["wizard", "sorcerer"]`.
    #[serde(default)]
    pub classes: Vec<String>,
    /// Casting time description, e.g. `"1 action"`.
    pub casting_time: String,
    /// Range description, e.g. `"120 feet"`.
    pub range: String,
    pub components: SpellComponents,
    /// Duration description, e.g. `"Instantaneous"`.
    pub duration: String,
    /// Whether the spell requires concentration.
    #[serde(default)]
    pub concentration: bool,
    /// Whether the spell can be cast as a ritual.
    #[serde(default)]
    pub ritual: bool,
    /// Rules text for the spell (1–3 sentences for the proof set).
    pub description: String,
    /// Whether the spell requires a spell attack roll, and if so whether melee or ranged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack: Option<SpellAttack>,
    /// Saving throw the target must make (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save: Option<SpellSave>,
    /// Damage (or healing) the spell deals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage: Option<SpellDamage>,
}

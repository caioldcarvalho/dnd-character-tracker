//! A [`Contribution`] is one traceable input to a stat: who added it
//! ([`SourceRef`]), how it combines ([`ContribOp`]), its [`ValueExpr`], an optional
//! activation [`Condition`], and an optional non-numeric d20 [`D20Effect`].

use crate::ids::{ClassId, StatId};
use crate::value::ValueExpr;
use serde::{Deserialize, Serialize};

/// How a contribution combines into its target stat. Applied in band order:
/// Base → Add → Multiply → Floor → Cap → Override.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContribOp {
    /// Establishes a base value. When several Bases compete, the **max** wins
    /// (D&D 2024 AC: armor vs Unarmored Defense are alternative calculations).
    Base,
    /// Adds to the running total (subject to stacking rules).
    Add,
    /// Multiplies the running total.
    Multiply,
    /// Raises the total to at least this value (a minimum/floor).
    Floor,
    /// Lowers the total to at most this value (a maximum/cap).
    Cap,
    /// Hard-sets the total, overriding everything.
    Override,
}

impl Default for ContribOp {
    fn default() -> Self {
        ContribOp::Add
    }
}

/// Non-numeric effects on a d20 test. Per 2024, one advantage cancels any number
/// of disadvantages and vice versa — they do not stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum D20Effect {
    Advantage,
    Disadvantage,
    AutoFail,
    AutoSuccess,
}

/// Gate for whether a contribution is currently active.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "if", rename_all = "kebab-case")]
pub enum Condition {
    Always,
    /// A toggle/active-effect with this id is on (e.g. `"rage"`).
    EffectActive { id: String },
    WearingArmor,
    /// Not wearing armor (e.g. Barbarian/Monk Unarmored Defense).
    Unarmored,
    WieldingShield,
}

/// Where a contribution came from — its provenance, shown in the breakdown.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "kebab-case")]
pub enum SourceRef {
    Base,
    Species { id: String },
    Class { class: ClassId, feature: String, level: u8 },
    Subclass { subclass: String, feature: String, level: u8 },
    Feat { id: String },
    Background { id: String },
    Item { id: String },
    Spell { id: String },
    Condition { id: String },
    Manual { label: String },
}

impl Default for SourceRef {
    fn default() -> Self {
        SourceRef::Base
    }
}

impl SourceRef {
    pub fn label(&self) -> String {
        match self {
            SourceRef::Base => "Base".into(),
            SourceRef::Species { id } => format!("Species: {id}"),
            SourceRef::Class { class, feature, .. } => format!("{class} — {feature}"),
            SourceRef::Subclass { subclass, feature, .. } => format!("{subclass} — {feature}"),
            SourceRef::Feat { id } => format!("Feat: {id}"),
            SourceRef::Background { id } => format!("Background: {id}"),
            SourceRef::Item { id } => format!("Item: {id}"),
            SourceRef::Spell { id } => format!("Spell: {id}"),
            SourceRef::Condition { id } => format!("Condition: {id}"),
            SourceRef::Manual { label } => label.clone(),
        }
    }
}

/// One traceable input to a stat.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contribution {
    pub target: StatId,
    #[serde(default)]
    pub op: ContribOp,
    #[serde(default)]
    pub value: ValueExpr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<Condition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<D20Effect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Contributions sharing a non-`None` group don't stack — only the largest applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_group: Option<String>,
}

impl Contribution {
    pub fn new(target: StatId, op: ContribOp, value: ValueExpr) -> Self {
        Self {
            target,
            op,
            value,
            when: None,
            effect: None,
            note: None,
            stack_group: None,
        }
    }
    pub fn add(target: StatId, value: ValueExpr) -> Self {
        Self::new(target, ContribOp::Add, value)
    }
    pub fn base(target: StatId, value: ValueExpr) -> Self {
        Self::new(target, ContribOp::Base, value)
    }
    pub fn cap(target: StatId, value: ValueExpr) -> Self {
        Self::new(target, ContribOp::Cap, value)
    }
    pub fn floor(target: StatId, value: ValueExpr) -> Self {
        Self::new(target, ContribOp::Floor, value)
    }
    pub fn d20(target: StatId, effect: D20Effect) -> Self {
        let mut c = Self::new(target, ContribOp::Add, ValueExpr::lit(0));
        c.effect = Some(effect);
        c
    }
    pub fn note(mut self, n: impl Into<String>) -> Self {
        self.note = Some(n.into());
        self
    }
    pub fn when(mut self, c: Condition) -> Self {
        self.when = Some(c);
        self
    }
    pub fn stacking(mut self, group: impl Into<String>) -> Self {
        self.stack_group = Some(group.into());
        self
    }
}

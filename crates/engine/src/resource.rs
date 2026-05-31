//! Spendable resources — one shape for every dice pool and point pool
//! (superiority dice, psi dice, sorcery points, ki, rage, channel divinity, lay
//! on hands, second wind, action surge, …). A resource's *max* is a [`ValueExpr`],
//! so it is as traceable as any other stat.

use crate::ids::ResourceId;
use crate::value::ValueExpr;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceKind {
    /// A pool of points (sorcery points, ki, lay-on-hands HP).
    Points,
    /// A pool of dice (superiority/psi/bardic dice).
    Dice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Recharge {
    ShortRest,
    LongRest,
    Dawn,
    Special,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceDef {
    pub id: ResourceId,
    pub name: String,
    pub kind: ResourceKind,
    /// Base die size for `Dice` pools (e.g. 8 for a d8).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub die: Option<u8>,
    /// Die size by character level (largest key `<= level` applies).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub die_scaling: Option<BTreeMap<u8, u8>>,
    /// Maximum size of the pool — a traceable expression.
    pub max: ValueExpr,
    pub recharge: Recharge,
}

impl ResourceDef {
    /// Effective die sides at a given character level.
    pub fn die_at(&self, level: u8) -> Option<u8> {
        match &self.die_scaling {
            Some(scaling) => scaling
                .range(..=level)
                .next_back()
                .map(|(_, &s)| s)
                .or(self.die),
            None => self.die,
        }
    }
}

/// A pool of hit dice of one size (multiclass characters have several).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitDiePool {
    pub sides: u8,
    pub total: u8,
    #[serde(default)]
    pub spent: u8,
}

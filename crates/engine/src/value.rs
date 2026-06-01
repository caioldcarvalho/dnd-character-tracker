//! [`ValueExpr`] — the recursive expression language a contribution's value is
//! written in. `Stat { id }` references another node, which is what turns the set
//! of stats into a dependency DAG.

use crate::ids::{Ability, StatId};
use serde::{Deserialize, Serialize};

/// A dice expression like `2d6`.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dice {
    pub count: u8,
    pub sides: u8,
}

impl Dice {
    pub fn new(count: u8, sides: u8) -> Self {
        Self { count, sides }
    }
    /// D&D "average, rounded up" per die = `sides/2 + 1` (d10 → 6, d8 → 5, d6 → 4).
    pub fn average_floor(self) -> i32 {
        self.count as i32 * (self.sides as i32 / 2 + 1)
    }
    pub fn max(self) -> i32 {
        self.count as i32 * self.sides as i32
    }
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "expr", rename_all = "kebab-case")]
pub enum ValueExpr {
    /// A constant.
    Literal { value: i32 },
    /// The value of another stat (the recursion / DAG edge).
    Stat { id: StatId },
    /// Shorthand for `floor((score - 10) / 2)`.
    AbilityMod { ability: Ability },
    Sum { terms: Vec<ValueExpr> },
    Product { terms: Vec<ValueExpr> },
    Min { terms: Vec<ValueExpr> },
    Max { terms: Vec<ValueExpr> },
    /// `per * by` (e.g. 5 × level for Lay on Hands).
    Scaled { per: Box<ValueExpr>, by: Box<ValueExpr> },
    /// Step function over `(threshold, value)` pairs: the value of the greatest
    /// threshold `<= on` applies. `on` defaults to character level. (Pairs, not a
    /// map, because integer map keys don't survive serde's tagged-enum buffering.)
    LevelTable {
        #[serde(default = "default_level_on")]
        on: Box<ValueExpr>,
        table: Vec<(u8, i32)>,
    },
    /// Euclidean floor division `num / den` (den clamped to >= 1).
    FloorDiv { num: Box<ValueExpr>, den: Box<ValueExpr> },
    /// The "average, rounded up" of a dice expression.
    DiceAverage { dice: Dice },
}

fn default_level_on() -> Box<ValueExpr> {
    Box::new(ValueExpr::Stat {
        id: StatId::CharacterLevel,
    })
}

impl Default for ValueExpr {
    fn default() -> Self {
        ValueExpr::Literal { value: 0 }
    }
}

// Ergonomic constructors for building expressions in Rust (tests, intrinsics).
impl ValueExpr {
    pub fn lit(value: i32) -> Self {
        ValueExpr::Literal { value }
    }
    pub fn stat(id: StatId) -> Self {
        ValueExpr::Stat { id }
    }
    pub fn ability_mod(ability: Ability) -> Self {
        ValueExpr::AbilityMod { ability }
    }
    pub fn sum(terms: Vec<ValueExpr>) -> Self {
        ValueExpr::Sum { terms }
    }
    pub fn product(terms: Vec<ValueExpr>) -> Self {
        ValueExpr::Product { terms }
    }
    pub fn min(terms: Vec<ValueExpr>) -> Self {
        ValueExpr::Min { terms }
    }
    pub fn max(terms: Vec<ValueExpr>) -> Self {
        ValueExpr::Max { terms }
    }
    pub fn scaled(per: ValueExpr, by: ValueExpr) -> Self {
        ValueExpr::Scaled {
            per: Box::new(per),
            by: Box::new(by),
        }
    }
    pub fn floor_div(num: ValueExpr, den: ValueExpr) -> Self {
        ValueExpr::FloorDiv {
            num: Box::new(num),
            den: Box::new(den),
        }
    }
    pub fn level_table(pairs: impl IntoIterator<Item = (u8, i32)>) -> Self {
        ValueExpr::LevelTable {
            on: default_level_on(),
            table: pairs.into_iter().collect(),
        }
    }
    pub fn level_table_on(on: ValueExpr, pairs: impl IntoIterator<Item = (u8, i32)>) -> Self {
        ValueExpr::LevelTable {
            on: Box::new(on),
            table: pairs.into_iter().collect(),
        }
    }
    pub fn dice_avg(count: u8, sides: u8) -> Self {
        ValueExpr::DiceAverage {
            dice: Dice::new(count, sides),
        }
    }
}

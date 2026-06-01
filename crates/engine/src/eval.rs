//! The evaluator. Resolves any [`StatId`] by reducing its contributions, with
//! memoization and 3-color cycle detection. `explain` runs the *same* reduction
//! while recording each line, so the breakdown can never disagree with the total.

use crate::build::{Built, ContribMap, EnvFlags, Sourced};
use crate::contribution::{Condition, ContribOp, D20Effect};
use crate::error::{EvalError, EvalErrorKind};
use crate::ids::StatId;
use crate::value::ValueExpr;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq)]
enum Color {
    Gray,
    Black,
}

/// One line of a stat's breakdown.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BreakdownLine {
    pub source: String,
    pub op: ContribOp,
    pub band: &'static str,
    pub value: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// False when this line was dropped (e.g. lost the Base max, or a non-stacking dupe).
    pub applied: bool,
}

/// The full provenance of a single stat.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StatBreakdown {
    pub stat: StatId,
    pub label: String,
    pub total: i32,
    pub lines: Vec<BreakdownLine>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceNote {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvState {
    Advantage,
    Disadvantage,
    Flat,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutoOutcome {
    Fail,
    Success,
    None,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct D20Channel {
    pub advantage: Vec<SourceNote>,
    pub disadvantage: Vec<SourceNote>,
    pub auto_fail: Vec<SourceNote>,
    pub auto_success: Vec<SourceNote>,
}

/// The resolved result of a d20 test: the numeric bonus + its breakdown, the
/// advantage state, and any auto outcome.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct D20Test {
    pub total: i32,
    pub breakdown: StatBreakdown,
    pub adv: AdvState,
    pub channel: D20Channel,
    pub auto: AutoOutcome,
}

pub struct EvalCtx<'a> {
    map: &'a ContribMap,
    active: &'a BTreeSet<String>,
    env: EnvFlags,
    memo: RefCell<BTreeMap<StatId, i32>>,
    color: RefCell<BTreeMap<StatId, Color>>,
    errors: RefCell<Vec<EvalError>>,
}

impl<'a> EvalCtx<'a> {
    pub fn new(built: &'a Built) -> Self {
        Self {
            map: &built.contribs,
            active: &built.active,
            env: built.env,
            memo: RefCell::new(BTreeMap::new()),
            color: RefCell::new(BTreeMap::new()),
            errors: RefCell::new(Vec::new()),
        }
    }

    pub fn eval(&self, id: &StatId) -> i32 {
        self.eval_stat(id)
    }

    pub fn take_errors(&self) -> Vec<EvalError> {
        std::mem::take(&mut self.errors.borrow_mut())
    }

    pub fn explain(&self, id: &StatId) -> StatBreakdown {
        if let StatId::AbilityModifier(_) = id {
            let total = self.eval_stat(id);
            return StatBreakdown {
                stat: id.clone(),
                label: id.label(),
                total,
                lines: vec![BreakdownLine {
                    source: "Derived".into(),
                    op: ContribOp::Base,
                    band: "derived",
                    value: total,
                    note: Some("floor((score − 10) / 2)".into()),
                    applied: true,
                }],
            };
        }
        let active = self.gather(id);
        let (total, lines) = self.reduce(id, &active);
        StatBreakdown {
            stat: id.clone(),
            label: id.label(),
            total,
            lines,
        }
    }

    pub fn d20(&self, id: &StatId) -> D20Test {
        let breakdown = self.explain(id);
        let active = self.gather(id);
        let mut channel = D20Channel::default();
        for s in &active {
            if let Some(effect) = s.contrib.effect {
                let note = SourceNote {
                    source: s.source.label(),
                    note: s.contrib.note.clone(),
                };
                match effect {
                    D20Effect::Advantage => channel.advantage.push(note),
                    D20Effect::Disadvantage => channel.disadvantage.push(note),
                    D20Effect::AutoFail => channel.auto_fail.push(note),
                    D20Effect::AutoSuccess => channel.auto_success.push(note),
                }
            }
        }
        let adv = match (!channel.advantage.is_empty(), !channel.disadvantage.is_empty()) {
            (true, true) | (false, false) => AdvState::Flat,
            (true, false) => AdvState::Advantage,
            (false, true) => AdvState::Disadvantage,
        };
        let auto = if !channel.auto_fail.is_empty() {
            AutoOutcome::Fail
        } else if !channel.auto_success.is_empty() {
            AutoOutcome::Success
        } else {
            AutoOutcome::None
        };
        D20Test {
            total: breakdown.total,
            breakdown,
            adv,
            channel,
            auto,
        }
    }

    fn gather(&self, id: &StatId) -> Vec<&'a Sourced> {
        match self.map.get(id) {
            Some(v) => v
                .iter()
                .filter(|s| self.cond(&s.contrib.when))
                .collect(),
            None => Vec::new(),
        }
    }

    fn cond(&self, when: &Option<Condition>) -> bool {
        match when {
            None => true,
            Some(c) => match c {
                Condition::Always => true,
                Condition::EffectActive { id } => self.active.contains(id),
                Condition::WearingArmor => self.env.wearing_armor,
                Condition::Unarmored => !self.env.wearing_armor,
                Condition::WieldingShield => self.env.wielding_shield,
            },
        }
    }

    fn eval_stat(&self, id: &StatId) -> i32 {
        if let Some(v) = self.memo.borrow().get(id) {
            return *v;
        }
        if let Some(Color::Gray) = self.color.borrow().get(id) {
            self.errors.borrow_mut().push(EvalError {
                stat: id.clone(),
                kind: EvalErrorKind::Cycle,
            });
            return 0;
        }
        self.color.borrow_mut().insert(id.clone(), Color::Gray);

        let val = match id {
            StatId::AbilityModifier(a) => {
                let score = self.eval_stat(&StatId::AbilityScore(*a));
                (score - 10).div_euclid(2)
            }
            _ => {
                let active = self.gather(id);
                self.reduce(id, &active).0
            }
        };

        self.color.borrow_mut().insert(id.clone(), Color::Black);
        self.memo.borrow_mut().insert(id.clone(), val);
        val
    }

    fn eval_value(&self, e: &ValueExpr) -> i32 {
        match e {
            ValueExpr::Literal { value } => *value,
            ValueExpr::Stat { id } => self.eval_stat(id),
            ValueExpr::AbilityMod { ability } => self.eval_stat(&StatId::AbilityModifier(*ability)),
            ValueExpr::Sum { terms } => terms.iter().map(|t| self.eval_value(t)).sum(),
            ValueExpr::Product { terms } => terms.iter().map(|t| self.eval_value(t)).product(),
            ValueExpr::Min { terms } => terms.iter().map(|t| self.eval_value(t)).min().unwrap_or(0),
            ValueExpr::Max { terms } => terms.iter().map(|t| self.eval_value(t)).max().unwrap_or(0),
            ValueExpr::Scaled { per, by } => self.eval_value(per) * self.eval_value(by),
            ValueExpr::LevelTable { on, table } => {
                let n = self.eval_value(on).clamp(0, 255) as u8;
                table
                    .iter()
                    .filter(|(at, _)| *at <= n)
                    .max_by_key(|(at, _)| *at)
                    .map(|(_, v)| *v)
                    .unwrap_or(0)
            }
            ValueExpr::FloorDiv { num, den } => {
                let d = self.eval_value(den).max(1);
                self.eval_value(num).div_euclid(d)
            }
            ValueExpr::DiceAverage { dice } => dice.average_floor(),
        }
    }

    /// The op-precedence band reducer. Produces both the total and the breakdown.
    fn reduce(&self, id: &StatId, active: &[&'a Sourced]) -> (i32, Vec<BreakdownLine>) {
        let mut lines = Vec::new();
        let line = |s: &Sourced, band: &'static str, value: i32, applied: bool| BreakdownLine {
            source: s.source.label(),
            op: s.contrib.op,
            band,
            value,
            note: s.contrib.note.clone(),
            applied,
        };

        // Band 0 — BASE: the max of competing bases wins.
        let bases: Vec<(i32, &Sourced)> = active
            .iter()
            .filter(|s| s.contrib.op == ContribOp::Base)
            .map(|s| (self.eval_value(&s.contrib.value), *s))
            .collect();
        let chosen_base = bases.iter().map(|(v, _)| *v).max();
        let mut acc = chosen_base.unwrap_or_else(|| default_base(id));
        let mut base_marked = false;
        for (v, s) in &bases {
            let applied = !base_marked && Some(*v) == chosen_base;
            if applied {
                base_marked = true;
            }
            lines.push(line(s, "base", *v, applied));
        }

        // Band 1 — ADD: sum, honoring non-stacking groups (largest wins per group).
        let adds: Vec<(i32, &Sourced)> = active
            .iter()
            .filter(|s| s.contrib.op == ContribOp::Add)
            .map(|s| (self.eval_value(&s.contrib.value), *s))
            .collect();
        let mut best: BTreeMap<String, i32> = BTreeMap::new();
        for (v, s) in &adds {
            if let Some(g) = &s.contrib.stack_group {
                let e = best.entry(g.clone()).or_insert(i32::MIN);
                if *v > *e {
                    *e = *v;
                }
            }
        }
        let mut used: BTreeSet<String> = BTreeSet::new();
        for (v, s) in &adds {
            let applied = match &s.contrib.stack_group {
                None => true,
                Some(g) => {
                    if *v == best[g] && !used.contains(g) {
                        used.insert(g.clone());
                        true
                    } else {
                        false
                    }
                }
            };
            if applied {
                acc += v;
            }
            lines.push(line(s, "add", *v, applied));
        }

        // Band 2 — MULTIPLY.
        for s in active.iter().filter(|s| s.contrib.op == ContribOp::Multiply) {
            let v = self.eval_value(&s.contrib.value);
            acc *= v;
            lines.push(line(s, "multiply", v, true));
        }

        // Band 3 — FLOOR then CAP.
        for s in active.iter().filter(|s| s.contrib.op == ContribOp::Floor) {
            let v = self.eval_value(&s.contrib.value);
            let applied = acc < v;
            if applied {
                acc = v;
            }
            lines.push(line(s, "floor", v, applied));
        }
        for s in active.iter().filter(|s| s.contrib.op == ContribOp::Cap) {
            let v = self.eval_value(&s.contrib.value);
            let applied = acc > v;
            if applied {
                acc = v;
            }
            lines.push(line(s, "cap", v, applied));
        }

        // Band 4 — OVERRIDE: the last one wins.
        if let Some(s) = active.iter().filter(|s| s.contrib.op == ContribOp::Override).last() {
            let v = self.eval_value(&s.contrib.value);
            acc = v;
            lines.push(line(s, "override", v, true));
        }

        (acc, lines)
    }
}

fn default_base(id: &StatId) -> i32 {
    match id {
        StatId::AbilityScore(_) => 10,
        _ => 0,
    }
}

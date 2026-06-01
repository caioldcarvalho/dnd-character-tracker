//! Rest mechanics (2024). A rest is a pure transform of the runtime state on a
//! [`CharacterSheet`]: it recharges resource pools per each pool's [`Recharge`]
//! rule, restores spell slots / hit dice, and (on a long rest) heals to full and
//! reduces exhaustion. The template/derived layers are untouched — `rest` only
//! edits the saved sheet's runtime fields.

use crate::build::build;
use crate::computed::compute;
use crate::content::ContentDb;
use crate::ids::{SlotLevel, StatId};
use crate::resource::Recharge;
use crate::sheet::CharacterSheet;

/// Which rest is being taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestKind {
    Short,
    Long,
}

impl RestKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "short" => Some(RestKind::Short),
            "long" => Some(RestKind::Long),
            _ => None,
        }
    }
}

/// Apply a rest, returning the updated sheet. Pure: no I/O.
///
/// 2024 rules modeled:
/// - **Short rest:** recharge `ShortRest` pools to full; spend Hit Dice to heal
///   is a *player* action (not automatic here), so HD are not auto-restored.
/// - **Long rest:** recharge `ShortRest` + `LongRest` + `Dawn` pools; restore all
///   spell slots; regain up to half your total Hit Dice; heal to full HP; drop
///   one level of exhaustion; clear temp HP and concentration.
pub fn rest(sheet: &CharacterSheet, content: &ContentDb, kind: RestKind) -> CharacterSheet {
    let mut out = sheet.clone();
    let built = build(sheet, content);

    // Recharge resource pools whose rule fires for this rest. A recharged pool is
    // simply removed from `resources` (the computed `current` then defaults to
    // its max), keeping the saved sheet minimal.
    for def in &built.resources {
        let fires = match (kind, def.recharge) {
            (RestKind::Short, Recharge::ShortRest) => true,
            (RestKind::Long, Recharge::ShortRest)
            | (RestKind::Long, Recharge::LongRest)
            | (RestKind::Long, Recharge::Dawn) => true,
            _ => false,
        };
        if fires {
            out.resources.remove(&def.id);
        }
    }

    if kind == RestKind::Long {
        // Spell slots: all restored.
        out.slots_expended.clear();
        // Note for callers/UI: SpellSlotMax is derived; clearing expenditure means
        // every slot reads full. (We touch the map only to be explicit.)
        let _ = StatId::SpellSlotMax(SlotLevel(1));

        // Hit Dice: regain up to half the total (rounded down, min 1 if any spent).
        let mut by_size: std::collections::BTreeMap<u8, u8> = std::collections::BTreeMap::new();
        for pool in &built.hit_dice {
            *by_size.entry(pool.sides).or_insert(0) += pool.total;
        }
        let total_hd: u32 = by_size.values().map(|&t| t as u32).sum();
        let mut regain = (total_hd / 2).max(1) as i32;
        // Restore largest dice first (favorable, and deterministic).
        let mut sizes: Vec<u8> = by_size.keys().copied().collect();
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        for size in sizes {
            if regain <= 0 {
                break;
            }
            let spent = out.hit_dice_spent.get(&size).copied().unwrap_or(0) as i32;
            if spent <= 0 {
                continue;
            }
            let give = spent.min(regain);
            let new_spent = (spent - give).max(0) as u8;
            if new_spent == 0 {
                out.hit_dice_spent.remove(&size);
            } else {
                out.hit_dice_spent.insert(size, new_spent);
            }
            regain -= give;
        }

        // Heal to full, clear temp HP, exhaustion −1, drop concentration.
        let computed = compute(&out, content);
        out.hp.current = computed.max_hp.total;
        out.hp.temp = 0;
        out.exhaustion = out.exhaustion.saturating_sub(1);
        out.concentration = None;
    }

    out
}

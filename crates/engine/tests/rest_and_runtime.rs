//! M2 runtime mechanics: rest recharge, spell-slot expenditure tracking, and
//! rolled-vs-average HP. All against the real content.

use rpgman_engine::*;
use std::path::Path;

fn content() -> ContentDb {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    ContentDb::load_dir(&root).expect("content/ should load")
}

/// A Battle Master Fighter 5 (has Second Wind = short-rest, superiority dice =
/// short-rest, and Indomitable-style long-rest pools at higher level).
fn fighter5() -> CharacterSheet {
    CharacterSheet::new("Tess")
        .with_class("fighter", 5, Some("battle-master"))
        .with_ability(Ability::Str, 16)
        .with_ability(Ability::Con, 14)
}

#[test]
fn rolled_hp_overrides_average() {
    let db = content();
    // Average: d10 → L1 max 10 + 4×avg(6) = 34 + CON(2)×5 = 44.
    let avg = compute(&fighter5(), &db).max_hp.total;
    assert_eq!(avg, 44);

    // Record max rolls (10 each) for levels 2..5 → 10 + 4×10 + 10 = 60.
    let mut rolled = fighter5();
    rolled.hp.rolled = vec![10, 10, 10, 10];
    let cc = compute(&rolled, &db);
    assert_eq!(cc.max_hp.total, 10 + 40 + 10, "rolled HP should beat averages");
    assert!(cc.max_hp.lines.iter().any(|l| l.note.as_deref() == Some("rolled d10")));
}

#[test]
fn short_rest_recharges_only_short_rest_pools() {
    let db = content();
    let mut sheet = fighter5();
    // Spend everything: superiority dice to 1, second wind to 0.
    sheet.resources.insert(ResourceId::new("superiority-dice"), 1);
    sheet.resources.insert(ResourceId::new("second-wind"), 0);

    let after = rest(&sheet, &db, RestKind::Short);
    let cc = compute(&after, &db);
    let sup = cc.resources.iter().find(|r| r.id.as_str().contains("superiority")).unwrap();
    let sw = cc.resources.iter().find(|r| r.id.as_str().contains("second-wind")).unwrap();
    assert_eq!(sup.current, sup.max, "superiority dice recharge on short rest");
    assert_eq!(sw.current, sw.max, "second wind recharges on short rest");
}

#[test]
fn long_rest_restores_slots_hp_and_exhaustion() {
    let db = content();
    // A Wizard 5 to exercise spell slots.
    let mut wiz = CharacterSheet::new("Mira")
        .with_class("wizard", 5, Some("evoker"))
        .with_ability(Ability::Int, 16)
        .with_ability(Ability::Con, 14);
    // Expend some slots, take damage, gain exhaustion, spend hit dice.
    wiz.slots_expended.insert(1, 4);
    wiz.slots_expended.insert(3, 2);
    wiz.hp.current = 5;
    wiz.hp.temp = 3;
    wiz.exhaustion = 2;
    wiz.hit_dice_spent.insert(6, 4);

    let max_hp = compute(&wiz, &db).max_hp.total;

    let after = rest(&wiz, &db, RestKind::Long);
    let cc = compute(&after, &db);

    // All slots full again.
    for s in &cc.spell_slots {
        assert_eq!(s.current, s.max, "L{} slots restored on long rest", s.level);
    }
    // HP to full, temp cleared, exhaustion down one.
    assert_eq!(after.hp.current, max_hp, "healed to full");
    assert_eq!(after.hp.temp, 0, "temp HP cleared");
    assert_eq!(after.exhaustion, 1, "exhaustion −1");
    // Regained up to half the hit dice (5 total → 2): spent 4 → 2.
    assert_eq!(after.hit_dice_spent.get(&6).copied().unwrap_or(0), 2, "regained half the HD");
}

#[test]
fn spell_slot_expenditure_is_tracked() {
    let db = content();
    let mut wiz = CharacterSheet::new("Mira")
        .with_class("wizard", 5, Some("evoker"))
        .with_ability(Ability::Int, 16);
    wiz.slots_expended.insert(1, 2);
    let cc = compute(&wiz, &db);
    let l1 = cc.spell_slots.iter().find(|s| s.level == 1).unwrap();
    assert_eq!(l1.max, 4);
    assert_eq!(l1.current, 2, "4 max − 2 expended = 2 available");
}

//! Replays the M2 play loop against the engine, mirroring the store's runtime
//! mutations: take damage, gain temp HP, spend resources and slots, then long
//! rest — asserting the engine restores everything per the 2024 rules. Headless.

use rpgman_engine::{compute, rest, CharacterSheet, ContentDb, ResourceId, RestKind};
use std::path::Path;

fn content() -> ContentDb {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../content");
    ContentDb::load_dir(&root).expect("content/ should load")
}

/// A Battle Master Fighter 5 with a wizard dip so we exercise slots too… actually
/// keep it a clean Cleric 5: short-rest channel divinity + full HP + slots.
fn cleric5() -> CharacterSheet {
    let mut s = CharacterSheet::new("Pike")
        .with_class("cleric", 5, Some("life-domain"))
        .with_ability(rpgman_engine::Ability::Con, 14)
        .with_ability(rpgman_engine::Ability::Wis, 16)
        .with_choice("cleric-divine-order", &["thaumaturge"]);
    let max = compute(&s, &content()).max_hp.total;
    s.hp.current = max;
    s
}

#[test]
fn damage_temp_and_heal_resolve_like_the_store() {
    let db = content();
    let mut s = cleric5();
    let max = compute(&s, &db).max_hp.total;

    // Store.applyDamage(5): no temp → current drops 5.
    s.hp.current = (s.hp.current - 5).max(0);
    assert_eq!(compute(&s, &db).current_hp, max - 5);

    // Store.setTempHp(8): temp set.
    s.hp.temp = 8;
    // Store.applyDamage(6): temp absorbs first (8→2), current untouched.
    {
        let mut dmg = 6;
        let absorbed = s.hp.temp.min(dmg);
        s.hp.temp -= absorbed;
        dmg -= absorbed;
        s.hp.current = (s.hp.current - dmg).max(0);
    }
    let cc = compute(&s, &db);
    assert_eq!(cc.temp_hp, 2, "temp HP absorbed the hit");
    assert_eq!(cc.current_hp, max - 5, "current HP untouched while temp absorbed");

    // Store.heal(100): clamps to max.
    s.hp.current = (s.hp.current + 100).min(max);
    assert_eq!(compute(&s, &db).current_hp, max);
}

#[test]
fn long_rest_restores_everything_after_a_hard_fight() {
    let db = content();
    let mut s = cleric5();
    let max = compute(&s, &db).max_hp.total;

    // A rough fight: spend channel divinity to 0, expend slots, take damage,
    // gain exhaustion.
    let cd_id = compute(&s, &db)
        .resources
        .iter()
        .find(|r| r.id.as_str().contains("channel"))
        .map(|r| r.id.clone())
        .expect("cleric has channel divinity");
    s.resources.insert(cd_id.clone(), 0);
    s.slots_expended.insert(1, 4);
    s.slots_expended.insert(2, 3);
    s.hp.current = 4;
    s.exhaustion = 1;

    // Long rest (the Rust authoritative path the store calls in Tauri).
    let after = rest(&s, &db, RestKind::Long);
    let cc = compute(&after, &db);

    // Channel divinity back to full.
    let cd = cc.resources.iter().find(|r| r.id == cd_id).unwrap();
    assert_eq!(cd.current, cd.max, "channel divinity recharged on long rest");
    // Slots full.
    for slot in &cc.spell_slots {
        assert_eq!(slot.current, slot.max, "L{} slots restored", slot.level);
    }
    // HP full, exhaustion cleared.
    assert_eq!(after.hp.current, max, "healed to full");
    assert_eq!(after.exhaustion, 0, "exhaustion 1 → 0");
}

#[test]
fn short_rest_leaves_long_rest_pools_spent() {
    let db = content();
    // Fighter: Second Wind (short) recharges, but spend a long-rest resource and
    // confirm short rest does NOT refill it.
    let mut s = CharacterSheet::new("Gar")
        .with_class("fighter", 9, Some("battle-master"))
        .with_ability(rpgman_engine::Ability::Con, 14);
    // Indomitable is long-rest at fighter 9.
    s.resources.insert(ResourceId::new("indomitable"), 0);
    s.resources.insert(ResourceId::new("second-wind"), 0);

    let after = rest(&s, &db, RestKind::Short);
    let cc = compute(&after, &db);
    let sw = cc.resources.iter().find(|r| r.id.as_str().contains("second-wind")).unwrap();
    let ind = cc.resources.iter().find(|r| r.id.as_str().contains("indomitable")).unwrap();
    assert_eq!(sw.current, sw.max, "second wind recharges on short rest");
    assert_eq!(ind.current, 0, "indomitable (long-rest) stays spent after a short rest");
}

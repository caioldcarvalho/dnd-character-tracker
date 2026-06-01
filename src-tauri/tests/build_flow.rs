//! Replays the M3 build flow against the engine, mirroring exactly what the UI's
//! store mutations do: start blank → set species/background/classes/abilities →
//! resolve every pending choice the engine reports, until none remain. Proves the
//! build pipeline produces a complete, correct character with no display needed.

use rpgman_engine::{compute, ContentDb};
use serde_json::{json, Value};
use std::path::Path;

fn content() -> ContentDb {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../content");
    ContentDb::load_dir(&root).expect("content/ should load")
}

/// A mutable JSON character sheet (the same shape the frontend store edits).
fn blank(name: &str) -> Value {
    json!({
        "meta": { "name": name, "player": "", "id": "" },
        "abilities": { "str": 10, "dex": 10, "con": 10, "int": 10, "wis": 10, "cha": 10 },
        "species": null,
        "background": null,
        "classes": [],
        "feats": [],
        "choices": [],
        "hp": { "current": 0, "temp": 0, "rolled": [] },
        "resources": {},
        "hit_dice_spent": {},
        "conditions": [],
        "exhaustion": 0,
        "active_effects": [],
        "equipment": { "armor": null, "shield": false },
        "weapons": [],
        "concentration": null,
        "death_saves": { "successes": 0, "failures": 0 },
        "inspiration": false
    })
}

fn compute_sheet(sheet: &Value, db: &ContentDb) -> rpgman_engine::ComputedCharacter {
    let parsed = serde_json::from_value(sheet.clone()).expect("sheet must deserialize");
    compute(&parsed, db)
}

/// Record a choice exactly like the store's resolveChoice for non-subclass kinds.
fn record_choice(sheet: &mut Value, key: &str, picks: &[&str]) {
    let arr = sheet["choices"].as_array_mut().unwrap();
    arr.retain(|c| c["key"] != key);
    arr.push(json!({ "key": key, "picks": picks }));
}

#[test]
fn build_a_fighter_from_scratch_resolving_every_choice() {
    let db = content();

    // 1. Start blank, then set identity + a class (store: newCharacter/addClass).
    let mut sheet = blank("Brienne");
    sheet["species"] = json!("human");
    sheet["background"] = json!("soldier");
    sheet["classes"] = json!([{ "class": "fighter", "level": 5, "subclass": null }]);
    sheet["abilities"] = json!({ "str": 15, "dex": 13, "con": 14, "int": 8, "wis": 12, "cha": 10 });

    // At this point the engine should report several pending choices, including a
    // subclass prompt (Fighter subclass at level 3).
    let cc = compute_sheet(&sheet, &db);
    assert!(
        cc.pending_choices.iter().any(|p| matches!(p.options, rpgman_engine::ChoiceOptions::Subclass)),
        "a level-5 Fighter with no subclass must prompt for one"
    );
    let initial_pending = cc.pending_choices.len();
    assert!(initial_pending > 0, "expected pending choices to resolve");

    // 2. Resolve the subclass like the store does (sets ClassEntry.subclass).
    sheet["classes"][0]["subclass"] = json!("battle-master");

    // 3. Drain the rest: loop, resolving each remaining pending choice with the
    //    first valid pick(s), exactly as a user clicking through the resolver.
    for _round in 0..12 {
        let cc = compute_sheet(&sheet, &db);
        let Some(choice) = cc.pending_choices.first().cloned() else {
            break;
        };
        let picks = first_valid_picks(&choice);
        assert!(
            !picks.is_empty(),
            "no pick strategy for choice kind {:?} (key {})",
            choice.options,
            choice.key
        );
        let refs: Vec<&str> = picks.iter().map(|s| s.as_str()).collect();
        record_choice(&mut sheet, &choice.key, &refs);
    }

    // 4. Every choice resolved → a complete character.
    let cc = compute_sheet(&sheet, &db);
    assert!(
        cc.pending_choices.is_empty(),
        "expected all choices resolved, still pending: {:?}",
        cc.pending_choices.iter().map(|p| &p.key).collect::<Vec<_>>()
    );
    assert!(cc.errors.is_empty(), "no eval errors: {:?}", cc.errors);
    assert_eq!(cc.level, 5);
    assert_eq!(cc.proficiency_bonus, 3);
    // Soldier +2 STR / +1 (we picked STR twice + CON below) lands; HP sane.
    assert!(cc.max_hp.total >= 38, "Fighter 5 HP should be healthy, got {}", cc.max_hp.total);
    // Battle Master gave a superiority-dice pool.
    assert!(
        cc.resources.iter().any(|r| r.id.as_str().contains("superiority")),
        "Battle Master must have superiority dice"
    );
}

/// Pick the first valid option(s) for a choice, mirroring the resolver UI.
fn first_valid_picks(choice: &rpgman_engine::PendingChoice) -> Vec<String> {
    use rpgman_engine::ChoiceOptions::*;
    let n = choice.choose as usize;
    match &choice.options {
        Skills { from } | Expertise { from } => from
            .iter()
            .take(n)
            .map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())
            .collect(),
        AbilityScoreIncrease => {
            // +1 to the first `choose` abilities (e.g. STR, DEX).
            ["str", "dex", "con", "int", "wis", "cha"]
                .iter()
                .take(n)
                .map(|s| s.to_string())
                .collect()
        }
        AbilityScores { from, points, .. } => {
            // Distribute `points` as +1 each across the first abilities.
            let ids: Vec<String> = from
                .iter()
                .map(|a| serde_json::to_value(a).unwrap().as_str().unwrap().to_string())
                .collect();
            let mut picks = Vec::new();
            for i in 0..*points as usize {
                picks.push(ids[i % ids.len()].clone());
            }
            picks
        }
        Feat { category } => {
            // The resolver would pick from catalog feats of this category; here we
            // hard-code a known origin feat for the soldier-style flow.
            match category.as_deref() {
                Some("fighting-style") => vec!["defense".into()],
                _ => vec!["tough".into()],
            }
        }
        Subclass => vec![], // handled out-of-band (set on ClassEntry)
        Spells { from, .. } | WeaponMastery { from } => {
            from.iter().take(n).cloned().collect()
        }
        Named { from } => from.iter().take(n).map(|o| o.id.clone()).collect(),
    }
}

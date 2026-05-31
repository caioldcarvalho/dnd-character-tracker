//! Cross-check every authored class: each sample character must LOAD through the
//! engine and produce the correct D&D 2024 math. Expected values are computed
//! independently here (not trusting the authoring agents).
//!
//! HP = max(die) + (floor(die/2)+1)*(level-1) + CON_mod*level [+ subclass HP].
//!
//! On failure, a one-line report is written to /tmp/all_classes_report.txt.

use rpgman_engine::*;
use std::path::Path;

fn content() -> ContentDb {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    ContentDb::load_dir(&root).expect("ALL content/ files must parse together")
}

struct Sample {
    class: &'static str,
    subclass: &'static str,
    abilities: Vec<(Ability, i32)>,
    choices: Vec<(&'static str, Vec<&'static str>)>,
    level: u8,
    expect_hp: i32,
    expect_save_dc: Option<i32>,
    expect_resources: Vec<(&'static str, i32)>,
}

fn sample(
    class: &'static str,
    subclass: &'static str,
    level: u8,
    abilities: Vec<(Ability, i32)>,
    choices: Vec<(&'static str, Vec<&'static str>)>,
    expect_hp: i32,
    expect_save_dc: Option<i32>,
    expect_resources: Vec<(&'static str, i32)>,
) -> Sample {
    Sample { class, subclass, abilities, choices, level, expect_hp, expect_save_dc, expect_resources }
}

fn build(s: &Sample) -> CharacterSheet {
    let mut sheet = CharacterSheet::new(s.class).with_class(s.class, s.level, Some(s.subclass));
    for (a, v) in &s.abilities {
        sheet = sheet.with_ability(*a, *v);
    }
    for (k, picks) in &s.choices {
        sheet = sheet.with_choice(*k, picks);
    }
    sheet
}

fn check(s: &Sample, db: &ContentDb, report: &mut String) {
    let cc = compute(&build(s), db);
    let mut fail = |msg: String| report.push_str(&format!("[{}] {}  ||  ", s.class, msg));

    if !cc.errors.is_empty() {
        fail(format!("eval errors: {:?}", cc.errors));
    }
    if cc.level != s.level as i32 {
        fail(format!("level {} != {}", cc.level, s.level));
    }
    if cc.max_hp.total != s.expect_hp {
        fail(format!("HP {} != expected {}", cc.max_hp.total, s.expect_hp));
    }
    if let Some(dc) = s.expect_save_dc {
        match cc.spellcasting.iter().find(|x| x.source == s.class) {
            Some(got) if got.save_dc == dc => {}
            Some(got) => fail(format!("save DC {} != expected {}", got.save_dc, dc)),
            None => fail(format!(
                "expected caster '{}', sources: {:?}",
                s.class,
                cc.spellcasting.iter().map(|x| x.source.clone()).collect::<Vec<_>>()
            )),
        }
    }
    for (needle, max) in &s.expect_resources {
        match cc.resources.iter().find(|r| r.id.as_str().contains(needle)) {
            Some(found) if found.max == *max => {}
            Some(found) => fail(format!("resource '{}' max {} != expected {}", needle, found.max, max)),
            None => fail(format!(
                "no resource matching '{}' (have: {:?})",
                needle,
                cc.resources.iter().map(|r| r.id.as_str().to_string()).collect::<Vec<_>>()
            )),
        }
    }
}

#[test]
fn every_authored_class_loads_and_computes() {
    use Ability::*;
    let db = content();

    let samples = vec![
        sample("cleric", "life-domain", 5,
            vec![(Str,10),(Dex,12),(Con,14),(Int,10),(Wis,16),(Cha,13)],
            vec![("cleric-divine-order", vec!["thaumaturge"]), ("cleric-skills", vec!["religion","insight"])],
            38, Some(14), vec![("channel-divinity", 2)]),
        sample("rogue", "thief", 5,
            vec![(Dex,16),(Con,14),(Int,13),(Wis,12)],
            vec![("rogue-expertise-1", vec!["stealth","sleight-of-hand"])],
            38, None, vec![]),
        sample("paladin", "oath-of-devotion", 5,
            vec![(Str,16),(Dex,10),(Con,14),(Wis,12),(Cha,16)],
            vec![("paladin-fighting-style-2", vec!["defense"])],
            44, Some(14), vec![("lay-on-hands", 25)]),
        sample("ranger", "hunter", 5,
            vec![(Str,12),(Dex,16),(Con,14),(Wis,14)],
            vec![("ranger-fighting-style", vec!["archery"])],
            44, Some(13), vec![]),
        sample("monk", "warrior-of-the-open-hand", 5,
            vec![(Str,12),(Dex,16),(Con,14),(Wis,15)],
            vec![],
            38, None, vec![("focus", 5)]),
        sample("sorcerer", "draconic-sorcery", 5,
            vec![(Dex,14),(Con,14),(Cha,16)],
            vec![],
            37, Some(14), vec![("sorcery-points", 5)]),
        sample("warlock", "fiend-patron", 5,
            vec![(Dex,14),(Con,14),(Cha,16)],
            vec![],
            38, Some(14), vec![]),
        sample("bard", "college-of-lore", 5,
            vec![(Dex,14),(Con,14),(Int,12),(Cha,16)],
            vec![("bard-expertise-1", vec!["persuasion","deception"])],
            38, Some(14), vec![("bardic", 3)]),
        sample("druid", "circle-of-the-moon", 5,
            vec![(Dex,14),(Con,14),(Int,12),(Wis,16)],
            vec![],
            38, Some(14), vec![("wild-shape", 2)]),
    ];

    let mut report = String::new();
    for s in &samples {
        check(s, &db, &mut report);
    }
    let _ = std::fs::write(
        "/tmp/all_classes_report.txt",
        if report.is_empty() { "ALL CLASSES OK" } else { &report },
    );
    assert!(report.is_empty(), "class mismatches written to /tmp/all_classes_report.txt");
}

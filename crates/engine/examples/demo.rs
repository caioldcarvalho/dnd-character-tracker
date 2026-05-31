//! A tiny console demo of the engine: load the rules-as-data files, build two
//! characters, and print their key stats *with provenance*.
//!
//! Run with:  cargo run -p rpgman-engine --example demo

use rpgman_engine::*;
use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    let content = ContentDb::load_dir(&root).expect("content/ should load");

    let aldric = {
        let mut s = CharacterSheet::new("Aldric")
            .with_class("fighter", 11, Some("battle-master"))
            .with_ability(Ability::Str, 16)
            .with_ability(Ability::Dex, 14)
            .with_ability(Ability::Con, 16)
            .with_ability(Ability::Wis, 12)
            .with_choice("fighter-fighting-style", &["defense"])
            .with_choice("battle-master-student-of-war", &["history"]);
        s.equipment.armor = Some(ArmorItem {
            name: "Chain Mail".into(),
            base_ac: 16,
            kind: ArmorKind::Heavy,
            dex_cap: Some(0),
        });
        s
    };

    let mira = CharacterSheet::new("Mira")
        .with_class("wizard", 5, Some("evoker"))
        .with_ability(Ability::Int, 16)
        .with_ability(Ability::Dex, 14)
        .with_ability(Ability::Con, 14)
        .with_ability(Ability::Wis, 12)
        .with_choice("wizard-skills", &["arcana", "investigation"]);

    // A multiclass + background + species character — all from data files.
    let thorin = CharacterSheet::new("Thorin")
        .with_species("dwarf")
        .with_background("soldier")
        .with_class("paladin", 6, Some("oath-of-devotion"))
        .with_class("sorcerer", 2, Some("draconic-sorcery"))
        .with_ability(Ability::Str, 16)
        .with_ability(Ability::Con, 14)
        .with_ability(Ability::Cha, 15)
        .with_choice("background-abilities", &["str", "str", "con"])
        .with_choice("paladin-fighting-style-2", &["defense"]);

    for sheet in [&aldric, &mira, &thorin] {
        print_character(&compute(sheet, &content));
    }
}

fn print_character(c: &ComputedCharacter) {
    println!("\n══════════════════════════════════════════");
    println!("  {}  (level {})", c.name, c.level);
    println!("══════════════════════════════════════════");

    print!("  Abilities: ");
    for a in &c.abilities {
        print!("{} {} ({:+})  ", a.ability_abbr(), a.score, a.modifier);
    }
    println!("\n  Proficiency Bonus: +{}", c.proficiency_bonus);

    print_breakdown("Armor Class", &c.armor_class);
    print_breakdown("Max HP", &c.max_hp);

    print!("\n  Saves: ");
    for s in &c.saves {
        let mark = if s.proficient { "*" } else { " " };
        print!("{}{}{:+}  ", s.ability_abbr(), mark, s.test.total);
    }
    println!("   (* = proficient)");

    if !c.spell_slots.is_empty() {
        print!("\n  Spell Slots: ");
        for s in &c.spell_slots {
            print!("L{}×{}  ", s.level, s.max);
        }
        println!();
        for sc in &c.spellcasting {
            println!(
                "  Spellcasting ({}): save DC {}, attack {:+}, prepared {}",
                sc.source,
                sc.save_dc,
                sc.attack_bonus,
                sc.prepared.map(|p| p.to_string()).unwrap_or_else(|| "—".into())
            );
        }
    }

    if !c.resources.is_empty() {
        println!("\n  Resources:");
        for r in &c.resources {
            let die = r.die.map(|d| format!(" (d{d})")).unwrap_or_default();
            println!("    • {}: {}/{}{}", r.name, r.current, r.max, die);
        }
    }

    let pending = c.pending_choices.len();
    if pending > 0 {
        println!("\n  Pending choices ({pending}):");
        for p in c.pending_choices.iter().take(6) {
            println!("    ? {} — {}", p.source, p.prompt);
        }
        if pending > 6 {
            println!("    … and {} more", pending - 6);
        }
    }
}

fn print_breakdown(label: &str, b: &StatBreakdown) {
    println!("\n  {} = {}", label, b.total);
    for line in &b.lines {
        if !line.applied {
            continue;
        }
        let note = line.note.as_deref().unwrap_or("");
        println!("      {:>4}  {:<22} {}", fmt_value(line.value), line.source, note);
    }
}

fn fmt_value(v: i32) -> String {
    if v >= 0 {
        format!("+{v}")
    } else {
        v.to_string()
    }
}

// Small helpers so the demo reads cleanly.
trait AbbrAbility {
    fn ability_abbr(&self) -> &'static str;
}
impl AbbrAbility for AbilityView {
    fn ability_abbr(&self) -> &'static str {
        self.ability.abbr()
    }
}
impl AbbrAbility for SaveView {
    fn ability_abbr(&self) -> &'static str {
        self.ability.abbr()
    }
}

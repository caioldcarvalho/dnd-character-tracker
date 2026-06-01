//! The rulebook as data. Classes, subclasses, species, and feats are authored as
//! JSON and loaded into a [`ContentDb`]. The engine never hardcodes a class —
//! adding one is adding a file.

use crate::error::EngineError;
use crate::feature::{ChoicePoint, Feature};
use crate::ids::{Ability, ClassId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

// Content baked into the binary at compile time by build.rs: a slice of
// (subdir, file-contents) pairs. Powers `ContentDb::embedded()`.
include!(concat!(env!("OUT_DIR"), "/embedded_content.rs"));

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Progression {
    Full,
    Half,
    Third,
    Pact,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Preparation {
    Prepared,
    Known,
    Spellbook,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CasterSpec {
    pub ability: Ability,
    pub progression: Progression,
    pub preparation: Preparation,
    /// Number of prepared spells by class level (2024 uses a fixed table). When
    /// absent, the UI can fall back to an ability-based formula.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepared_per_level: Option<BTreeMap<u8, i32>>,
}

/// What a class/subclass grants at one level.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct LevelGrant {
    #[serde(default)]
    pub features: Vec<Feature>,
    #[serde(default)]
    pub choices: Vec<ChoicePoint>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClassDef {
    pub id: ClassId,
    pub name: String,
    pub hit_die: u8,
    #[serde(default)]
    pub saving_throws: Vec<Ability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spellcasting: Option<CasterSpec>,
    #[serde(default = "default_subclass_level")]
    pub subclass_level: u8,
    /// Level (1..=20) → grants. Use a `BTreeMap` for stable, ordered iteration.
    #[serde(default)]
    pub levels: BTreeMap<u8, LevelGrant>,
}

fn default_subclass_level() -> u8 {
    3
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubclassDef {
    pub id: String,
    pub name: String,
    pub class: ClassId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spellcasting: Option<CasterSpec>,
    #[serde(default)]
    pub levels: BTreeMap<u8, LevelGrant>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpeciesDef {
    pub id: String,
    pub name: String,
    #[serde(default = "default_speed")]
    pub speed: i32,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub traits: Vec<Feature>,
}

fn default_speed() -> i32 {
    30
}

/// A 2024 background: ability-score increases, skill & tool proficiencies, and
/// an origin feat. (In 2024, ability scores come from the background, not the
/// species.)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BackgroundDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// The three abilities this background can boost (+2/+1 or +1/+1/+1).
    #[serde(default)]
    pub abilities: Vec<Ability>,
    /// Skill proficiencies granted.
    #[serde(default)]
    pub skills: Vec<crate::ids::Skill>,
    /// Tool proficiencies granted.
    #[serde(default)]
    pub tools: Vec<String>,
    /// The origin feat id this background grants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_feat: Option<String>,
}

/// The loaded rulebook. Keyed by id string for easy lookup and JSON-friendliness.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ContentDb {
    #[serde(default)]
    pub classes: BTreeMap<String, ClassDef>,
    #[serde(default)]
    pub subclasses: BTreeMap<String, SubclassDef>,
    #[serde(default)]
    pub species: BTreeMap<String, SpeciesDef>,
    #[serde(default)]
    pub backgrounds: BTreeMap<String, BackgroundDef>,
    #[serde(default)]
    pub feats: BTreeMap<String, Feature>,
}

impl ContentDb {
    pub fn with_class(mut self, c: ClassDef) -> Self {
        self.classes.insert(c.id.0.clone(), c);
        self
    }
    pub fn with_subclass(mut self, s: SubclassDef) -> Self {
        self.subclasses.insert(s.id.clone(), s);
        self
    }
    pub fn with_species(mut self, s: SpeciesDef) -> Self {
        self.species.insert(s.id.clone(), s);
        self
    }
    pub fn with_background(mut self, b: BackgroundDef) -> Self {
        self.backgrounds.insert(b.id.clone(), b);
        self
    }
    pub fn with_feat(mut self, f: Feature) -> Self {
        self.feats.insert(f.id.clone(), f);
        self
    }

    pub fn class(&self, id: &ClassId) -> Option<&ClassDef> {
        self.classes.get(&id.0)
    }
    pub fn subclass(&self, id: &str) -> Option<&SubclassDef> {
        self.subclasses.get(id)
    }
    pub fn species_def(&self, id: &str) -> Option<&SpeciesDef> {
        self.species.get(id)
    }
    pub fn background(&self, id: &str) -> Option<&BackgroundDef> {
        self.backgrounds.get(id)
    }
    pub fn feat(&self, id: &str) -> Option<&Feature> {
        self.feats.get(id)
    }

    /// Build the database from content embedded into the binary at compile time
    /// (see `build.rs`). This is what the packaged app uses when no external
    /// `content/` directory is present, so it ships self-contained.
    pub fn embedded() -> Result<Self, EngineError> {
        let mut db = ContentDb::default();
        for (kind, text) in EMBEDDED_CONTENT {
            match *kind {
                "classes" => insert_all(parse_text::<ClassDef>(text, kind)?, |c| {
                    db.classes.insert(c.id.0.clone(), c);
                }),
                "subclasses" => insert_all(parse_text::<SubclassDef>(text, kind)?, |s| {
                    db.subclasses.insert(s.id.clone(), s);
                }),
                "species" => insert_all(parse_text::<SpeciesDef>(text, kind)?, |s| {
                    db.species.insert(s.id.clone(), s);
                }),
                "backgrounds" => insert_all(parse_text::<BackgroundDef>(text, kind)?, |b| {
                    db.backgrounds.insert(b.id.clone(), b);
                }),
                "feats" => insert_all(parse_text::<Feature>(text, kind)?, |f| {
                    db.feats.insert(f.id.clone(), f);
                }),
                _ => {}
            }
        }
        Ok(db)
    }

    /// Load every `*.json` file under `root/{classes,subclasses,species,feats}`,
    /// each file holding one definition. Missing subdirectories are skipped.
    pub fn load_dir(root: &Path) -> Result<Self, EngineError> {
        let mut db = ContentDb::default();
        for class in read_jsons::<ClassDef>(&root.join("classes"))? {
            db.classes.insert(class.id.0.clone(), class);
        }
        for sc in read_jsons::<SubclassDef>(&root.join("subclasses"))? {
            db.subclasses.insert(sc.id.clone(), sc);
        }
        for sp in read_jsons::<SpeciesDef>(&root.join("species"))? {
            db.species.insert(sp.id.clone(), sp);
        }
        for bg in read_jsons::<BackgroundDef>(&root.join("backgrounds"))? {
            db.backgrounds.insert(bg.id.clone(), bg);
        }
        for ft in read_jsons::<Feature>(&root.join("feats"))? {
            db.feats.insert(ft.id.clone(), ft);
        }
        Ok(db)
    }
}

/// Parse one JSON string holding a single definition or an array of them.
fn parse_text<T: for<'de> Deserialize<'de>>(text: &str, label: &str) -> Result<Vec<T>, EngineError> {
    let mkerr = |e: serde_json::Error| EngineError::Parse {
        path: format!("<embedded {label}>"),
        source: e,
    };
    if text.trim_start().starts_with('[') {
        serde_json::from_str::<Vec<T>>(text).map_err(mkerr)
    } else {
        serde_json::from_str::<T>(text).map(|v| vec![v]).map_err(mkerr)
    }
}

fn insert_all<T>(items: Vec<T>, mut insert: impl FnMut(T)) {
    for item in items {
        insert(item);
    }
}

fn read_jsons<T: for<'de> Deserialize<'de>>(dir: &Path) -> Result<Vec<T>, EngineError> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(out), // optional directory
    };
    for entry in entries {
        let entry = entry.map_err(|e| EngineError::Io {
            path: dir.display().to_string(),
            message: e.to_string(),
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|e| EngineError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        // Parse straight from the string (not via `Value`) so serde_json's
        // integer map-key coercion works for our `BTreeMap<u8, _>` level tables.
        // A file may hold a single definition or an array of them.
        let mkerr = |e: serde_json::Error| EngineError::Parse {
            path: path.display().to_string(),
            source: e,
        };
        if text.trim_start().starts_with('[') {
            let items: Vec<T> = serde_json::from_str(&text).map_err(mkerr)?;
            out.extend(items);
        } else {
            out.push(serde_json::from_str(&text).map_err(mkerr)?);
        }
    }
    Ok(out)
}

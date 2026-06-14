//! Lightweight summaries of all loaded content, for the build-mode pickers. The
//! UI needs names and a few key facts without deserializing whole definitions.

use crate::content::{ContentDb, Progression};
use crate::ids::Ability;
use crate::spell::SpellDef;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassSummary {
    pub id: String,
    pub name: String,
    pub hit_die: u8,
    /// "full" | "half" | "third" | "pact" | "none" — caster progression, if any.
    pub caster: Option<String>,
    pub subclass_level: u8,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubclassSummary {
    pub id: String,
    pub name: String,
    pub class: String,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeciesSummary {
    pub id: String,
    pub name: String,
    pub speed: i32,
    pub size: String,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackgroundSummary {
    pub id: String,
    pub name: String,
    pub abilities: Vec<Ability>,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatSummary {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
}

/// Everything the build pickers need, in one payload.
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Catalog {
    pub classes: Vec<ClassSummary>,
    pub subclasses: Vec<SubclassSummary>,
    pub species: Vec<SpeciesSummary>,
    pub backgrounds: Vec<BackgroundSummary>,
    pub feats: Vec<FeatSummary>,
    /// Full spell definitions (the set is small; the UI needs descriptions/range/etc.).
    pub spells: Vec<SpellDef>,
}

fn progression_label(p: Progression) -> Option<String> {
    match p {
        Progression::Full => Some("full".into()),
        Progression::Half => Some("half".into()),
        Progression::Third => Some("third".into()),
        Progression::Pact => Some("pact".into()),
        Progression::None => None,
    }
}

impl Catalog {
    /// Build the catalog from a loaded content database.
    pub fn from_content(db: &ContentDb) -> Self {
        let classes = db
            .classes
            .values()
            .map(|c| ClassSummary {
                id: c.id.0.clone(),
                name: c.name.clone(),
                hit_die: c.hit_die,
                caster: c.spellcasting.as_ref().and_then(|s| progression_label(s.progression)),
                subclass_level: c.subclass_level,
            })
            .collect();
        let subclasses = db
            .subclasses
            .values()
            .map(|s| SubclassSummary {
                id: s.id.clone(),
                name: s.name.clone(),
                class: s.class.0.clone(),
            })
            .collect();
        let species = db
            .species
            .values()
            .map(|s| SpeciesSummary {
                id: s.id.clone(),
                name: s.name.clone(),
                speed: s.speed,
                size: s.size.clone(),
            })
            .collect();
        let backgrounds = db
            .backgrounds
            .values()
            .map(|b| BackgroundSummary {
                id: b.id.clone(),
                name: b.name.clone(),
                abilities: b.abilities.clone(),
            })
            .collect();
        let feats = db
            .feats
            .values()
            .map(|f| FeatSummary {
                id: f.id.clone(),
                name: f.name.clone(),
                category: f.category.clone(),
            })
            .collect();
        let spells = db.spells.values().cloned().collect();
        Catalog {
            classes,
            subclasses,
            species,
            backgrounds,
            feats,
            spells,
        }
    }
}

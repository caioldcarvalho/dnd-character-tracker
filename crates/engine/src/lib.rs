//! rpgman-engine — the D&D 2024 character rules engine.
//!
//! Design tenet: **the app is just math.** Every character stat is a named node
//! ([`StatId`]) whose value is the reduction of a list of [`Contribution`]s, each
//! carrying its own *source*, *operation*, *value*, and optional *condition*. A
//! contribution's value can reference another stat ([`ValueExpr::Stat`]), so stats
//! form a dependency DAG. This means we can always answer "what is affecting this
//! value?" — the breakdown is data, not a forgotten formula.
//!
//! Rules live in data ([`ContentDb`]); a [`CharacterSheet`] records build choices +
//! runtime state; [`compute`] interprets them into a [`ComputedCharacter`] with a
//! full breakdown for every stat.

pub mod build;
pub mod catalog;
pub mod computed;
pub mod content;
pub mod contribution;
pub mod error;
pub mod eval;
pub mod feature;
pub mod ids;
pub mod resource;
pub mod rest;
pub mod sheet;
pub mod spell;
pub mod value;

pub use build::{build, Built, EnvFlags, PendingChoice, ResolvedFeature, SpellSource};
pub use catalog::{
    BackgroundSummary, Catalog, ClassSummary, FeatSummary, SpeciesSummary, SubclassSummary,
};
pub use computed::{
    compute, explain, AbilityView, ComputedCharacter, EffectView, PassiveView, ResourceView,
    SaveView, SkillView, SpeedView, SpellSlotView, SpellcastingView, WeaponView,
};
pub use content::{
    BackgroundDef, CasterSpec, ClassDef, ContentDb, LevelGrant, Preparation, Progression,
    SpeciesDef, SubclassDef,
};
pub use contribution::{Condition, ContribOp, Contribution, D20Effect, SourceRef};
pub use error::{EngineError, EvalError, EvalErrorKind};
pub use eval::{AdvState, AutoOutcome, BreakdownLine, D20Channel, D20Test, EvalCtx, StatBreakdown};
pub use feature::{
    Activation, ChoiceOptions, ChoicePoint, Feature, NamedOption, ProficiencyGrant, SpellGrant,
};
pub use ids::{
    Ability, CastingSource, ClassId, MovementKind, ResourceId, Skill, SlotLevel, StatId, WeaponKind,
};
pub use resource::{HitDiePool, Recharge, ResourceDef, ResourceKind};
pub use rest::{rest, RestKind};
pub use sheet::{
    ArmorItem, ArmorKind, CharacterSheet, ClassEntry, DeathSaves, Equipment, HpState, Meta,
    RecordedChoice, WeaponInstance,
};
pub use spell::{SpellAttack, SpellComponents, SpellDamage, SpellDef, SpellSave, SpellSchool};
pub use value::{Dice, ValueExpr};

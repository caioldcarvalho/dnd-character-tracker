//! Errors. Evaluation never panics on authoring bugs — it records an
//! [`EvalError`] and substitutes a safe default so the UI can surface the problem.

use crate::ids::StatId;
use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvalErrorKind {
    /// A dependency cycle was detected while resolving this stat.
    Cycle,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EvalError {
    pub stat: StatId,
    pub kind: EvalErrorKind,
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("io error at {path}: {message}")]
    Io { path: String, message: String },
    #[error("failed to parse {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}

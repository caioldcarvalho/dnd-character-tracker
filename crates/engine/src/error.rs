//! Errors. Evaluation never panics on authoring bugs — it records an
//! [`EvalError`] and substitutes a safe default so the UI can surface the problem.

use crate::ids::StatId;
use serde::Serialize;
use thiserror::Error;

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvalErrorKind {
    /// A dependency cycle was detected while resolving this stat.
    Cycle,
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
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

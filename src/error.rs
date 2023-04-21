use rhai::{EvalAltResult, Position};
use std::{
    cell::{BorrowError, BorrowMutError},
    error::Error,
    path::PathBuf,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MendaxError {
    #[error("unknown field {field:?}, expected one of: {}", .expected.join(", "))]
    UnknownField {
        field: String,
        expected: Vec<&'static str>,
    },

    #[error("system calls are forbidden at this sandbox level")]
    SystemForbidden,

    #[error("unknown colour {0:?}, expected one of: {}", .1.join(", "))]
    UnknownColour(String, &'static [&'static str]),

    #[error("^C")]
    KeyboardInterrupt,

    #[error("cannot nest screens")]
    NestedScreens,

    #[error("ambiguous source: both {f1} and {f2} exist")]
    AmbiguousSource { f1: String, f2: String },

    #[error("could not read file '{stem}' or '{stem}.rhai'", stem=stem.display())]
    NoSuchSource {
        stem: PathBuf,
        error: Box<dyn Error>,
    },

    #[error("lie in use")]
    LieUnreadable {
        error: Box<BorrowError>,
        at: Option<Position>,
    },

    #[error("lie in use")]
    LieUnwritable {
        error: Box<BorrowMutError>,
        at: Option<Position>,
    },

    #[error("tag '{name}' defined multiple times")]
    DuplicateTag { name: String },
}

impl From<MendaxError> for EvalAltResult {
    fn from(value: MendaxError) -> Self {
        EvalAltResult::ErrorSystem("mendax error".into(), Box::new(value))
    }
}

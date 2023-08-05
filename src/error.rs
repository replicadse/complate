use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("std: {0}")]
    Std(#[from] Box<dyn std::error::Error+Sync+Send>),
    #[error("generic: {0}")]
    Generic(String),
    // #[error("many")]
    // Many(Vec<Self>),
    #[error("argument: {0}")]
    Argument(String),
    // #[error("experimental command")]
    // ExperimentalCommand,
    #[error("unknown command")]
    UnknownCommand,
    #[error("no trust: {0}")]
    NoTrust(String),
    #[error("shell command: {0}")]
    ShellCommand(String),
    #[error("user interact abort")]
    InteractAbort,
    #[error("invalid: {0}")]
    Invalid(String),
}

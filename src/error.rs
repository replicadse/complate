use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("std")]
    Std(#[from] Box<dyn std::error::Error + Sync + Send>),
    #[error("generic")]
    Generic(String),
    // #[error("many")]
    // Many(Vec<Self>),
    #[error("argument")]
    Argument(String),
    #[error("experimental command")]
    ExperimentalCommand,
    #[error("unknown command")]
    UnknownCommand,
    #[error("no trust")]
    NoTrust,
    #[error("shell command")]
    ShellCommand(String),
    #[error("user interact abort")]
    InteractAbort,
    #[error("helper")]
    Helper(String),
}

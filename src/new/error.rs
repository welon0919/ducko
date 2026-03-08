use rustyline::error::ReadlineError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error(transparent)]
    ReadlineError(#[from] ReadlineError),
    #[error("Init cancelled")]
    InitCancelled,
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
}

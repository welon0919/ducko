use rustyline::error::ReadlineError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error(transparent)]
    ReadlineError(#[from] ReadlineError),
    #[error("Init cancelled")]
    InitCancelled,
    #[error("Directory {0} already exist")]
    DirectoryAlreadyExist(String),
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
}

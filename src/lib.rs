mod build;
mod config;
mod metadata;
mod serve;

use std::fmt::Display;

pub use build::build;
pub use serve::serve;

const PORT: u16 = 3333;

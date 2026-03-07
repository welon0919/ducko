mod build;
mod metadata;
mod serve;

use std::fmt::Display;

pub use build::build;
pub use serve::{serve, watch_files};

const PORT: u16 = 3333;

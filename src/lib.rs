mod build;
mod metadata;
mod serve;

use std::{fmt::Display, net::SocketAddr};

use axum::Router;
pub use build::build;
use log::{debug, info};
pub use serve::serve;
use tower_http::services::ServeDir;

use crate::build::OUTPUT_PATH;
const PORT: u16 = 3333;

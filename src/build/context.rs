use serde::{Deserialize, Serialize};

use crate::metadata::PostMetadata;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostContext {
    pub meta: PostMetadata,
    pub url: String,
}

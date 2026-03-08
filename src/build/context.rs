use serde::{Deserialize, Serialize};

use crate::metadata::PostMetadata;

// Context for post used in template
#[derive(Debug, Serialize, Deserialize)]
pub struct PostContext {
    pub meta: PostMetadata,
    pub url: String,
}

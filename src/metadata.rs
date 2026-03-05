use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PostMetadata {
    title: String,
    date: String,
    #[serde(default)]
    tags: Vec<String>,
}

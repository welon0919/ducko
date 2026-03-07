use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PostMetadata {
    title: String,
    date: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    template: Option<String>,
}
impl PostMetadata {
    pub(crate) fn get_template(&self) -> Option<&str> {
        self.template.as_deref()
    }
}

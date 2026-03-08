use std::{collections::HashMap, default::Default};

use crate::config::{Author, SiteConfig};

#[derive(Default)]
pub struct SiteConfigBuilder {
    title: String,
    description: String,
    base_url: String,
    author: Author,
}

impl SiteConfigBuilder {
    pub fn new() -> Self {
        Self {
            title: "My blog".to_owned(),
            ..Default::default()
        }
    }
    pub fn title(self, title: String) -> Self {
        Self { title, ..self }
    }
    pub fn description(self, description: String) -> Self {
        Self {
            description,
            ..self
        }
    }
    pub fn base_url(self, base_url: String) -> Self {
        Self { base_url, ..self }
    }
    pub fn author_name(self, author_name: String) -> Self {
        let author = Author {
            name: author_name,
            ..self.author
        };
        Self { author, ..self }
    }
    pub fn author_email(self, email: String) -> Self {
        let author = Author {
            email: Some(email),
            ..self.author
        };
        Self { author, ..self }
    }
    pub fn build(self) -> SiteConfig {
        SiteConfig {
            title: self.title,
            description: self.description,
            base_url: self.base_url,
            language_code: "en".to_string(),
            author: self.author,
            menu: vec![],
            extra: HashMap::new(),
        }
    }
}

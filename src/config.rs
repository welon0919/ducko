mod error;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::error::ConfigError;

pub const CONFIG_PATH: &str = "config.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub base_url: String,
    #[serde(default = "default_language")]
    pub language_code: String,
    pub author: Author,
    pub menu: Vec<MenuItem>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub name: String,
    pub url: String,
}
impl SiteConfig {
    pub fn load_config() -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(CONFIG_PATH).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ConfigError::ConfigFileNotFound(CONFIG_PATH.to_string())
            } else {
                e.into()
            }
        })?;
        let config: SiteConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

fn default_language() -> String {
    // TODO load default language
    "en".to_string()
}

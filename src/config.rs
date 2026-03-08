pub(crate) mod error;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::error::ConfigError;

pub const CONFIG_PATH: &str = "config.yaml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// The global config for the site
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

/// The author data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
}
/// An item in the menu
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MenuItem {
    pub name: String,
    pub url: String,
}
impl SiteConfig {
    /// Loads the default config from `config.yaml`
    /// # Errors
    /// Will return `Err` if:
    /// 1. The config file is not found
    /// 2. It lacks the permission to read the config file
    /// 3. The config file cannot be parsed
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

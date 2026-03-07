use thiserror::Error;
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config file {0} not found")]
    ConfigFileNotFound(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    ConfigParseError(#[from] serde_yaml::Error),
}

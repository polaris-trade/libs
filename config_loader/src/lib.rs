use serde::{Deserialize, Serialize};

pub mod database;
pub mod kafka;
pub mod loader;
pub mod logging;
pub mod redis;
pub use loader::{HttpSource, load_config, load_config_async};

// re-export for convenience
pub use config::{Config, ConfigBuilder, ConfigError, Environment, File, FileFormat};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BaseAppConfig {
    pub name: String,
    pub version: Option<String>,
    pub env: Option<String>,
    /// timezone offset in hours from UTC (e.g., 7 for UTC+7)
    pub timezone: Option<i8>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Env {
    #[serde(rename = "dev")]
    Development,
    Staging,
    Production,
    Unknown(String),
}

impl From<String> for Env {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "development" | "dev" | "sit" => Env::Development,
            "staging" | "stg" => Env::Staging,
            "production" | "prod" => Env::Production,
            other => Env::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RemoteConfig {
    pub config: _RemoteConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct _RemoteConfig {
    pub url: String,
}

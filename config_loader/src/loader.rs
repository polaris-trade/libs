use async_trait::async_trait;
use config::{
    AsyncSource, Config, ConfigBuilder, ConfigError, File, FileFormat, FileStoredFormat, Format,
    Map, Value, ValueKind, builder::AsyncState,
};
use serde::de::DeserializeOwned;
use std::{
    fmt::Debug,
    io::{Error, ErrorKind},
    path::PathBuf,
};

#[derive(Debug)]
pub struct HttpSource<F: Format> {
    uri: String,
    format: F,
}

#[async_trait]
impl<F: Format + Send + Sync + Debug> AsyncSource for HttpSource<F> {
    async fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        reqwest::get(&self.uri)
            .await
            .map_err(|e| ConfigError::Foreign(Box::new(e)))? // error conversion is possible from custom AsyncSource impls
            .text()
            .await
            .map_err(|e| ConfigError::Foreign(Box::new(e)))
            .and_then(|text| {
                self.format
                    .parse(Some(&self.uri), &text)
                    .map_err(ConfigError::Foreign)
            })
    }
}

pub fn load_config<T>(path: &str) -> Result<T, ConfigError>
where
    T: serde::de::DeserializeOwned,
{
    let mut config_path = PathBuf::from(path);

    config_path =
        std::fs::canonicalize(&config_path).map_err(|e| ConfigError::Foreign(Box::new(e)))?;

    let settings = Config::builder()
        .add_source(File::from(config_path))
        .build()?;

    settings
        .try_deserialize::<T>()
        .map_err(|e| ConfigError::Foreign(Box::new(e)))
}

/// Load configuration asynchronously from a remote HTTP endpoint
pub async fn load_config_async<T>(uri: &str, format: FileFormat) -> Result<T, ConfigError>
where
    T: DeserializeOwned + Send,
{
    let config = ConfigBuilder::<AsyncState>::default()
        .add_async_source(HttpSource {
            uri: uri.into(),
            format,
        })
        .build()
        .await?;

    config.try_deserialize()
}

#[derive(Debug, Clone)]
pub struct PropertiesFile;

impl Format for PropertiesFile {
    fn parse(
        &self,
        uri: Option<&String>,
        text: &str,
    ) -> Result<Map<String, Value>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = Map::new();

        for (lineno, line) in text.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments (# or !)
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }

            // Split key=value
            let (key, value) = match line.split_once('=') {
                Some((k, v)) => (k.trim(), v.trim()),
                None => {
                    return Err(Box::new(Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid line {}: '{}'", lineno + 1, line),
                    )));
                }
            };

            result.insert(
                key.to_string(),
                Value::new(uri, ValueKind::String(value.to_string())),
            );
        }

        Ok(result)
    }
}

impl FileStoredFormat for PropertiesFile {
    fn file_extensions(&self) -> &'static [&'static str] {
        &["properties"]
    }
}

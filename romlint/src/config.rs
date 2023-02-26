use crate::error::{Error, Result};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use tokio::fs::read_to_string;
use toml::from_str;

pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
    let s = read_to_string(path).await.map_err(Error::Io)?;
    from_str(s.as_str()).map_err(|err| Error::Deserialize(Box::new(err)))
}

#[derive(Debug, Deserialize)]
pub struct Config {
    archive_formats: Vec<String>,
    db_dir: String,
    #[serde(rename = "system")]
    systems: HashMap<String, SystemConfig>,
}

#[derive(Debug, Deserialize)]
pub struct SystemConfig {
    archive_format: String,
    obsolete_formats: Vec<String>,
}

impl Config {
    pub fn resolve(&self, system: &str) -> Option<ResolvedConfig<'_>> {
        self.systems.get(system).map(|sys| ResolvedConfig {
            archive_formats: self.archive_formats.iter().map(|s| s.as_str()).collect(),
            archive_format: sys.archive_format.as_str(),
            obsolete_formats: sys.obsolete_formats.iter().map(|s| s.as_str()).collect(),
        })
    }
}

pub struct ResolvedConfig<'a> {
    pub archive_formats: Vec<&'a str>,
    pub archive_format: &'a str,
    pub obsolete_formats: Vec<&'a str>,
}

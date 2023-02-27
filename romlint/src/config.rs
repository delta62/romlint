use crate::error::{ConfigReadErr, IoErr, Result};
use serde::Deserialize;
use snafu::prelude::*;
use std::{collections::HashMap, path::Path};
use tokio::fs::read_to_string;
use toml::from_str;

pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
    let s = read_to_string(path.as_ref()).await.context(IoErr {
        path: path.as_ref(),
    })?;
    from_str(s.as_str()).context(ConfigReadErr {})
}

#[derive(Debug, Deserialize)]
pub struct Config {
    global: GlobalConfig,
    #[serde(rename = "system")]
    systems: HashMap<String, SystemConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    archive_formats: Vec<String>,
    db_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct SystemConfig {
    archive_format: String,
    obsolete_formats: Option<Vec<String>>,
    raw_format: String,
}

impl Config {
    pub fn resolve(&self, system: &str) -> Option<ResolvedConfig<'_>> {
        self.systems.get(system).map(|sys| ResolvedConfig {
            archive_formats: self
                .global
                .archive_formats
                .iter()
                .map(|s| s.as_str())
                .collect(),
            raw_format: sys.raw_format.as_str(),
            archive_format: sys.archive_format.as_str(),
            obsolete_formats: sys
                .obsolete_formats
                .as_ref()
                .map(|fmts| fmts.iter().map(|s| s.as_str()).collect()),
        })
    }

    pub fn db_dir(&self) -> &str {
        self.global.db_dir.as_str()
    }
}

pub struct ResolvedConfig<'a> {
    pub archive_formats: Vec<&'a str>,
    pub archive_format: &'a str,
    pub obsolete_formats: Option<Vec<&'a str>>,
    pub raw_format: &'a str,
}

use crate::error::{ConfigReadErr, IoErr, Result};
use mlua::IntoLua;
use serde::{
    de::{self, value::SeqAccessDeserializer, Visitor},
    Deserialize, Deserializer,
};
use snafu::prelude::*;
use std::fmt::{self, Formatter};
use std::{collections::HashMap, marker::PhantomData, path::Path};
use tokio::fs::read_to_string;
use toml::from_str;

#[derive(Debug, Deserialize)]
pub struct Config {
    global: GlobalConfig,
    #[serde(rename = "system")]
    systems: HashMap<String, SystemConfig>,
}

impl Config {
    pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let s = read_to_string(path.as_ref()).await.context(IoErr {
            path: path.as_ref(),
        })?;

        from_str(s.as_str()).context(ConfigReadErr)
    }
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    db_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct SystemConfig {
    #[serde(deserialize_with = "string_or_vec")]
    archive_format: Vec<String>,
    obsolete_formats: Option<Vec<String>>,
    #[serde(deserialize_with = "string_or_vec")]
    raw_format: Vec<String>,
}

impl Config {
    pub fn resolve(&self, system: &str) -> Option<ResolvedConfig<'_>> {
        self.systems.get(system).map(|sys| ResolvedConfig {
            raw_format: sys.raw_format.iter().map(|s| s.as_str()).collect(),
            archive_format: sys.archive_format.iter().map(|s| s.as_str()).collect(),
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
    pub archive_format: Vec<&'a str>,
    pub obsolete_formats: Option<Vec<&'a str>>,
    pub raw_format: Vec<&'a str>,
}

impl<'cfg, 'lua> IntoLua<'lua> for &ResolvedConfig<'cfg> {
    fn into_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("archive_format", self.archive_format.clone())?;
        table.set(
            "obsolete_formats",
            self.obsolete_formats.clone().unwrap_or_default(),
        )?;
        table.set("raw_format", self.raw_format.clone())?;

        Ok(mlua::Value::Table(table))
    }
}

/// Deserialize either a string or a list of strings to `Vec<String>`. In the case of a string as
/// input, a singleton `Vec` will be returned containing that string.
fn string_or_vec<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<fn() -> String>);

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("string or array")
        }

        fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![v])
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![v.to_string()])
        }

        fn visit_seq<A>(self, seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            Deserialize::deserialize(SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}

use crate::{
    db::{Database, Databases},
    filemeta::{ArchiveInfo, FileMeta},
    word_match::Tokens,
};
use bitflags::bitflags;
use futures::io;
use rlua::{Function, Lua, Result, StdLib, ToLua, Value};
use std::{
    collections::HashMap, fs::Metadata, os::unix::prelude::MetadataExt, path::Path as FsPath,
    sync::Arc,
};
use tokio::fs::read_to_string;

pub struct ScriptLoader {
    scripts: Vec<Script>,
}

impl ScriptLoader {
    pub fn new() -> Self {
        let scripts = vec![];
        Self { scripts }
    }

    pub async fn load<P: AsRef<FsPath>>(&mut self, path: P) -> io::Result<()> {
        let src = read_to_string(&path).await?;
        let name = path
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_owned();
        let requirements = Self::get_requirements(&src, &name).unwrap();

        log::debug!("Loading script: {name}");

        self.scripts.push(Script {
            requirements,
            src,
            name,
        });

        Ok(())
    }

    pub fn requirements(&self) -> Requirements {
        self.scripts
            .iter()
            .fold(Requirements::empty(), |acc, x| acc | x.requirements)
    }

    fn get_requirements(src: &str, name: &str) -> Result<Requirements> {
        let lua = Lua::new_with(StdLib::BASE | StdLib::TABLE | StdLib::STRING);
        lua.context(|ctx| {
            ctx.load(src).set_name(name)?.exec()?;

            let reqs = ctx
                .globals()
                .get::<&str, Vec<String>>("requires")?
                .into_iter()
                .fold(Requirements::empty(), |acc, r| match r.as_str() {
                    "stat" => acc | Requirements::STAT,
                    "path" => acc | Requirements::PATH,
                    "archive" => acc | Requirements::ARCHIVE,
                    "file_db" => acc | Requirements::FILE_DB,
                    s => {
                        log::warn!("Unknown requirement listed: '{s}'");
                        acc
                    }
                });

            Ok(reqs)
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Script> {
        self.scripts.iter()
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Requirements: u32 {
        const PATH    = 0b0001;
        const STAT    = 0b0010;
        const ARCHIVE = 0b0100;
        const FILE_DB = 0b1000;
    }
}

impl Requirements {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::PATH => "path",
            Self::STAT => "stat",
            Self::ARCHIVE => "archive",
            Self::FILE_DB => "file_db",
            _ => "multiple requirements",
        }
    }
}

struct Stat {
    is_dir: bool,
    is_file: bool,
    mode: u32,
}

impl<'lua> ToLua<'lua> for Stat {
    fn to_lua(self, lua: rlua::Context<'lua>) -> Result<rlua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("mode", self.mode & 0o777)?;
        table.set("is_dir", self.is_dir)?;
        table.set("is_file", self.is_file)?;

        Ok(rlua::Value::Table(table))
    }
}

pub struct Script {
    requirements: Requirements,
    src: String,
    name: String,
}

impl std::fmt::Debug for Script {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn exec_one(script: &Script, meta: &FileMeta, databases: Arc<Databases>) -> Result<()> {
    // Each (file,script) pair gets an isolated Lua context to run in
    let lua = Lua::new_with(StdLib::BASE | StdLib::TABLE | StdLib::STRING);

    lua.context(|ctx| {
        ctx.scope(|scope| {
            let globals = ctx.globals();
            let file = ctx.create_table()?;
            let api = ctx.create_table()?;

            let assert_eq = ctx.create_function(
                |_, (expected, actual, detail): (Value, Value, Option<String>)| {
                    if lua_eq(&expected, &actual) {
                        return Ok(());
                    }

                    let err = AssertionError::expected(&expected, &actual, detail);
                    let err = rlua::Error::ExternalError(Arc::new(err));
                    Err(err)
                },
            )?;

            let assert_ne = ctx.create_function(
                |_, (not_expected, actual, detail): (Value, Value, Option<String>)| {
                    if lua_eq(&not_expected, &actual) {
                        let err = AssertionError::unexpected(&not_expected, detail);
                        let err = rlua::Error::ExternalError(Arc::new(err));
                        Err(err)?
                    }

                    Ok(())
                },
            )?;

            let throw = ctx.create_function(|_, detail: String| -> Result<()> {
                let err = AssertionError::throw(detail);
                let err = rlua::Error::ExternalError(Arc::new(err));
                Err(err)
            })?;

            let assert_contains = ctx.create_function(
                |_, (haystack, needle, detail): (Vec<Value>, Value, Option<String>)| {
                    for item in haystack {
                        if lua_eq(&item, &needle) {
                            return Ok(());
                        }
                    }

                    let detail = detail.unwrap_or_else(|| {
                        format!("Couldn't find '{}' in collection", fmt_lua(&needle),)
                    });

                    let err = AssertionError::with_message(detail);
                    let err = rlua::Error::ExternalError(Arc::new(err));
                    Err(err)
                },
            )?;

            api.set("assert_eq", assert_eq)?;
            api.set("assert_ne", assert_ne)?;
            api.set("assert_contains", assert_contains)?;
            api.set("throw", throw)?;

            api.set("system", meta.system())?;
            api.set("config", meta.config())?;

            let db_contains = scope.create_function(|_, ()| {
                let has_db_requirement = script.requirements.contains(Requirements::FILE_DB);

                if !has_db_requirement {
                    let err = RequirementError::new(Requirements::FILE_DB);
                    let err = rlua::Error::ExternalError(Arc::new(err));
                    Err(err)?;
                }

                let stem = meta.path().file_stem().and_then(|s| s.to_str());
                let result = match stem {
                    Some(stem) => meta
                        .system()
                        .and_then(|sys| databases.as_ref().get(sys))
                        .map(|db| db.contains(stem))
                        .unwrap_or_default(),
                    None => false,
                };

                Ok(result)
            })?;

            api.set("db_contains", db_contains)?;

            let similar_files = scope.create_function(|_, ()| {
                let has_db_requirement = script.requirements.contains(Requirements::FILE_DB);
                if !has_db_requirement {
                    let err = RequirementError::new(Requirements::FILE_DB);
                    let err = rlua::Error::ExternalError(Arc::new(err));
                    Err(err)?;
                }

                let db = meta.system().and_then(|sys| databases.as_ref().get(sys));
                let path_str = meta.path().to_str();

                match (db, path_str) {
                    (Some(db), Some(path)) => {
                        let file_tokens = Tokens::from_str(path);
                        let similar_titles = db.similar_to(&file_tokens);
                        let similar = similar_titles
                            .into_iter()
                            .map(|title| title.to_string())
                            .collect::<Vec<_>>();
                        Ok(similar)
                    }
                    _ => Ok(vec![]),
                }
            })?;

            api.set("similar_files", similar_files)?;

            let stat = scope.create_function(|_, ()| {
                if !script.requirements.contains(Requirements::STAT) {
                    let err = RequirementError::new(Requirements::STAT);
                    let err = rlua::Error::ExternalError(Arc::new(err));
                    Err(err)?;
                }

                let stat: Stat = meta.metadata().into();
                Ok(stat)
            })?;

            file.set("stat", stat)?;

            let path = scope.create_function(|_, ()| {
                if !script.requirements.contains(Requirements::PATH) {
                    let err = RequirementError::new(Requirements::PATH);
                    let err = rlua::Error::ExternalError(Arc::new(err));
                    Err(err)?;
                }

                let path: Path = meta.path().into();
                Ok(path)
            })?;

            file.set("path", path)?;

            let archive = scope.create_function(|_, ()| {
                if !script.requirements.contains(Requirements::ARCHIVE) {
                    let err = RequirementError::new(Requirements::ARCHIVE);
                    let err = rlua::Error::ExternalError(Arc::new(err));
                    Err(err)?;
                }

                let archive: Archive = meta.archive().into();
                Ok(archive)
            })?;

            file.set("archive", archive)?;

            ctx.load(&script.src).set_name(&script.name)?.exec()?;
            globals.get::<&str, Function>("lint")?.call((file, api))
        })
    })
}

struct Archive {
    files: Option<Vec<String>>,
}

impl<'lua> ToLua<'lua> for Archive {
    fn to_lua(self, lua: rlua::Context<'lua>) -> Result<Value<'lua>> {
        let table = lua.create_table()?;

        table.set("files", self.files)?;

        Ok(Value::Table(table))
    }
}

impl From<Option<&ArchiveInfo>> for Archive {
    fn from(value: Option<&ArchiveInfo>) -> Self {
        let files = value.map(|a| {
            a.file_names()
                .map(|path| path.to_str().unwrap_or("").to_string())
                .collect()
        });

        Self { files }
    }
}

struct Path {
    extension: Option<String>,
    stem: Option<String>,
    path: String,
}

impl<'lua> ToLua<'lua> for Path {
    fn to_lua(self, lua: rlua::Context<'lua>) -> Result<Value<'lua>> {
        let table = lua.create_table()?;

        table.set("extension", self.extension)?;
        table.set("stem", self.stem)?;
        table.set("path", self.path)?;

        Ok(rlua::Value::Table(table))
    }
}

impl From<&FsPath> for Path {
    fn from(value: &FsPath) -> Self {
        Self {
            extension: value
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_string()),
            stem: value
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| stem.to_string()),
            path: value
                .to_str()
                .map(|p| p.to_string())
                .unwrap_or_else(String::new),
        }
    }
}

impl<'a> From<&'a Metadata> for Stat {
    fn from(value: &'a Metadata) -> Self {
        Self {
            is_dir: value.is_dir(),
            is_file: value.is_file(),
            mode: value.mode(),
        }
    }
}

#[derive(Debug)]
struct RequirementError {
    wanted: Requirements,
}

impl RequirementError {
    pub fn new(wanted: Requirements) -> Self {
        Self { wanted }
    }
}

impl std::fmt::Display for RequirementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let req_str = self.wanted.to_str();
        write!(
            f,
            "Requested {req_str} data, but this capability was not declared"
        )
    }
}

impl std::error::Error for RequirementError {}

#[derive(Debug)]
struct AssertionError {
    message: String,
}

impl AssertionError {
    pub fn expected(expected: &Value, actual: &Value, detail: Option<String>) -> Self {
        let expected = fmt_lua(expected);
        let actual = fmt_lua(actual);
        let message = detail.unwrap_or_else(|| format!("expected {expected} to equal {actual}"));

        Self { message }
    }

    pub fn unexpected(unexpected: &Value, detail: Option<String>) -> Self {
        let unexpected = fmt_lua(unexpected);
        let detail = detail.as_deref().unwrap_or("assertion error");

        let message = format!("{detail} - unexpected value '{unexpected}'");
        Self { message }
    }

    pub fn throw<S: Into<String>>(message: S) -> Self {
        let message = message.into();
        Self { message }
    }

    pub fn with_message<S: Into<String>>(message: S) -> Self {
        let message = message.into();
        Self { message }
    }
}

impl std::fmt::Display for AssertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

fn fmt_lua(value: &Value) -> String {
    match value {
        Value::Nil => "nil".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::String(s) => s.to_str().unwrap_or("<non-displayable string>").to_string(),
        Value::Table(_) => "[table]".to_string(),
        Value::Function(_) => "[function]".to_string(),
        Value::Error(e) => format!("Error ({e})"),
        Value::Thread(_) => "[thread]".to_string(),
        Value::LightUserData(_) => "[light_userdata]".to_string(),
        Value::UserData(_) => "[userdata]".to_string(),
    }
}

fn lua_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::String(a), Value::String(b)) => a
            .to_str()
            .and_then(|a| b.to_str().map(|b| (a, b)))
            .map(|(a, b)| a == b)
            .unwrap_or(false),
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        _ => todo!(),
    }
}

impl std::error::Error for AssertionError {}

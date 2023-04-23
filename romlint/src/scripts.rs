use bitflags::bitflags;
use futures::io;
use rlua::{Function, Lua, StdLib, ToLua, Value};
use std::{fs::Metadata, os::unix::prelude::MetadataExt, path::Path, sync::Arc};
use tokio::fs::read_to_string;

use crate::filemeta::FileMeta;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Requirements: u32 {
        const PATH = 0b0001;
        const STAT = 0b0010;
    }
}

impl Requirements {
    pub fn to_str(&self) -> &'static str {
        match self {
            &Self::PATH => "path",
            &Self::STAT => "stat",
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
    fn to_lua(self, lua: rlua::Context<'lua>) -> rlua::Result<rlua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("mode", self.mode & 0o777)?;
        table.set("is_dir", self.is_dir)?;
        table.set("is_file", self.is_file)?;

        Ok(rlua::Value::Table(table))
    }
}

struct Script {
    requirements: Requirements,
    src: String,
    name: String,
}

pub struct ScriptHost {
    scripts: Vec<Script>,
}

impl ScriptHost {
    pub fn new() -> Self {
        let scripts = vec![];
        Self { scripts }
    }

    pub async fn load<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let src = read_to_string(&path).await?;
        let name = path
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_owned();
        let requirements = Self::get_requirements(&src, &name).unwrap();

        self.scripts.push(Script {
            requirements,
            src,
            name,
        });

        Ok(())
    }

    fn get_requirements(src: &str, name: &str) -> rlua::Result<Requirements> {
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
                    s => {
                        log::warn!("Unknown requirement listed: '{s}'");
                        acc
                    }
                });

            Ok(reqs)
        })
    }

    pub fn exec_all<'a>(&'a self, meta: &'a FileMeta) -> rlua::Result<()> {
        log::debug!("Executing lints for {:?}", meta.path());
        let lua = Lua::new_with(StdLib::BASE | StdLib::TABLE | StdLib::STRING);

        lua.context(|ctx| {
            ctx.scope(|scope| {
                let script = self.scripts.first().unwrap();

                let globals = ctx.globals();
                let file = ctx.create_table()?;
                let api = ctx.create_table()?;

                let assert_eq = ctx.create_function(
                    |_, (expected, actual, detail): (Value, Value, Option<String>)| {
                        if lua_eq(&expected, &actual) {
                            return Ok(());
                        }

                        let err = AssertionError::new(&expected, &actual, detail);
                        let err = rlua::Error::ExternalError(Arc::new(err));
                        Err(err)
                    },
                )?;

                api.set("assert_eq", assert_eq)?;

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

                ctx.load(&script.src).set_name(&script.name)?.exec()?;
                let lint: Function = globals.get("lint")?;
                lint.call((file, api))
            })
        })
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
    pub fn new(expected: &Value, actual: &Value, detail: Option<String>) -> Self {
        let expected = fmt_lua(&expected);
        let actual = fmt_lua(&actual);
        let detail = detail
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("assertion error");

        let message = format!("{detail} - expected {expected} to equal {actual}");
        Self { message }
    }
}

impl<'a> std::fmt::Display for AssertionError {
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

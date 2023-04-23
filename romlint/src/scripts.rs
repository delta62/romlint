use bitflags::bitflags;
use futures::io;
use rlua::{Lua, StdLib, ToLua};
use std::{
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
};
use tokio::fs::read_to_string;

use crate::filemeta::FileMeta;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Requirements: u32 {
        const PATH = 0b0001;
        const STAT = 0b0010;
    }
}

struct Stat {
    is_dir: bool,
    is_file: bool,
    mode: u32,
}

impl<'lua> ToLua<'lua> for Stat {
    fn to_lua(self, lua: rlua::Context<'lua>) -> rlua::Result<rlua::Value<'lua>> {
        let mut table = lua.create_table()?;
        table.set("mode", self.mode);
        table.set("is_dir", self.is_dir);
        table.set("is_file", self.is_file);

        Ok(rlua::Value::Table(table))
    }
}

struct Script {
    requirements: Requirements,
    src: String,
    name: String,
}

struct ScriptHost {
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
        let requirements = Requirements::STAT;

        self.scripts.push(Script {
            requirements,
            src,
            name,
        });

        Ok(())
    }

    pub fn exec_all<'a>(&'a self, file: &'a FileMeta) -> rlua::Result<()> {
        log::debug!("Executing lints for {:?}", file.path());
        let lua = Lua::new_with(StdLib::BASE | StdLib::TABLE | StdLib::STRING);

        lua.context(|ctx| {
            ctx.scope(|scope| {
                let stat = scope.create_function(|_ctx, ()| {
                    Ok(Stat {
                        is_dir: file.metadata().is_dir(),
                        is_file: file.metadata().is_file(),
                        mode: file.metadata().mode(),
                    })
                })?;
                ctx.globals().set("stat", stat)?;
                log::debug!("Lua context initialized");

                let script = self.scripts.first().unwrap();
                ctx.load(&script.src).set_name(&script.name)?.exec()
            })
        })
    }
}

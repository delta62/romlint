use crate::args::{Args, LintArgs};
use crate::commands::{check, scan};
use crate::config;
use crate::db::{self, Database};
use crate::error::{BrokenPipeErr, IoErr, Result};
use crate::filemeta::FileMeta;
use crate::scripts::{Requirements, Script, ScriptLoader};
use crate::ui::{Message, Summary, Ui};
use snafu::ResultExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use std::{sync::mpsc, thread::spawn};
use tokio::fs::read_dir;

pub struct LintContext {
    cwd: PathBuf,
    scripts: ScriptLoader,
}

impl LintContext {
    pub fn new(cwd: PathBuf) -> Self {
        let scripts = ScriptLoader::new();
        Self { cwd, scripts }
    }

    pub fn relative_path<P: AsRef<Path>>(&self, path: P) -> Option<&Path> {
        path.as_ref().strip_prefix(self.cwd.as_path()).ok()
    }

    pub fn scripts(&self) -> impl Iterator<Item = &Script> {
        self.scripts.iter()
    }

    pub fn databases(&self) -> Arc<HashMap<String, Database>> {
        todo!()
    }
}

pub async fn lint(args: &Args, lint_args: &LintArgs) -> Result<()> {
    let config_path = args.config_path();
    let config = config::from_path(config_path).await?;
    let db_path = args.cwd().join(config.db_dir());
    let (tx, rx) = mpsc::channel();
    let mut script_loader = ScriptLoader::new();

    let mut dir = read_dir("./lints").await.unwrap();
    while let Ok(Some(file)) = dir.next_entry().await {
        script_loader.load(file.path()).await.unwrap();
    }

    let hide_passes = lint_args.hide_passes;
    let ui_thread = spawn(move || Ui::new(rx, !hide_passes).run());
    let databases = if let Some(sys) = &args.system {
        db::load_only(&db_path, &[sys.as_str()], Some(&tx)).await?
    } else {
        db::load_all(&db_path, Some(&tx)).await?
    };
    let databases = Arc::new(databases);
    let on_message = |message: Message| tx.send(message).context(BrokenPipeErr {});

    if let Some(file) = lint_args.file.as_ref() {
        let start_time = Instant::now();
        let mut summary = Summary::new(start_time);
        let read_archives = script_loader.requirements().contains(Requirements::ARCHIVE);
        let cwd = args.cwd();
        let system = args.system.as_deref();
        let file = FileMeta::from_path(system, &config, file, read_archives)
            .await
            .context(IoErr { path: file })?;
        let passed = check(ctx, &file, on_message)?;

        if passed {
            summary.add_success(system.unwrap());
        } else {
            summary.add_failure(system.unwrap());
        }

        summary.mark_ended();
        tx.send(Message::Finished(summary))
            .context(BrokenPipeErr {})?;
    } else {
        scan(&args, &config, &script_loader, tx, databases).await?;
    }

    ui_thread.join().unwrap()?;

    Ok(())
}
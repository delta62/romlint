use crate::args::{Args, LintArgs, Reporter};
use crate::commands::{check, scan};
use crate::config::{self, Config};
use crate::db::{self, Databases};
use crate::error::{BrokenPipeErr, IoErr, Result};
use crate::filemeta::{Extractor, FileMeta, ZipExtractor};
use crate::scripts::{Requirements, Script, ScriptLoader};
use crate::ui::{AnsiReporter, JsonReporter, Message, Summary, Ui};
use snafu::ResultExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use std::{sync::mpsc, thread::spawn};
use tokio::fs::read_dir;

pub struct LintContext {
    config: Config,
    cwd: PathBuf,
    databases: Arc<Databases>,
    scripts: ScriptLoader,
    system: Option<String>,
}

impl LintContext {
    pub fn new(
        cwd: PathBuf,
        system: Option<&String>,
        databases: Databases,
        config: Config,
        scripts: ScriptLoader,
    ) -> Self {
        let system = system.cloned();
        let databases = Arc::new(databases);

        Self {
            config,
            cwd,
            databases,
            scripts,
            system,
        }
    }

    pub fn relative_path<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        let path = path.as_ref();
        path.strip_prefix(self.cwd.as_path())
            .map(|p| p.to_path_buf())
            .ok()
    }

    pub fn scripts(&self) -> impl Iterator<Item = &Script> {
        self.scripts.iter()
    }

    pub fn script_requirements(&self) -> Requirements {
        self.scripts.requirements()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn system(&self) -> Option<&String> {
        self.system.as_ref()
    }

    pub fn scan_dirs(&self) -> PathBuf {
        match &self.system {
            Some(system) => self.cwd.join(system),
            None => self.cwd.clone(),
        }
    }

    pub fn should_read_archives(&self) -> bool {
        self.scripts.requirements().contains(Requirements::ARCHIVE)
    }

    pub fn databases(&self) -> Arc<Databases> {
        self.databases.clone()
    }
}

pub async fn lint(args: &Args, lint_args: &LintArgs) -> Result<()> {
    let config_path = args.config_path();
    let config = config::from_path(config_path).await?;
    let (tx, rx) = mpsc::channel();
    let mut script_loader = ScriptLoader::new();

    let mut dir = read_dir("./lints").await.unwrap();
    while let Ok(Some(file)) = dir.next_entry().await {
        script_loader.load(file.path()).await.unwrap();
    }

    let hide_passes = lint_args.hide_passes;
    let reporter: Box<dyn crate::ui::Reporter + Send + Sync> = match lint_args.reporter {
        Reporter::Ansi => Box::new(AnsiReporter::new(!hide_passes)),
        Reporter::Json => Box::new(JsonReporter::new()),
    };
    let ui_thread = spawn(move || Ui::new(rx, reporter).run());
    let on_message = |message: Message| tx.send(message).context(BrokenPipeErr);

    let db_path = args.cwd().join(config.db_dir());
    let databases = if let Some(sys) = &args.system {
        db::load_only(&db_path, &[sys.as_str()], &on_message)
            .await
            .unwrap()
    } else {
        db::load_all(&db_path, &on_message).await.unwrap()
    };

    let ctx = LintContext::new(
        args.cwd(),
        args.system.as_ref(),
        databases,
        config,
        script_loader,
    );

    if let Some(file) = lint_args.file.as_ref() {
        let start_time = Instant::now();
        let mut extractors = HashMap::<String, Box<dyn Extractor>>::new();
        let mut summary = Summary::new(start_time);

        let read_archives = ctx.script_requirements().contains(Requirements::ARCHIVE);
        if read_archives {
            extractors.insert("zip".to_string(), Box::new(ZipExtractor));
        }

        let system = args.system.as_deref();
        let file = FileMeta::from_path(system, ctx.config(), file, &extractors)
            .await
            .context(IoErr { path: file })?;

        let passed = check(&ctx, &file, on_message)?;

        if passed {
            summary.add_success(system.unwrap());
        } else {
            summary.add_failure(system.unwrap());
        }

        summary.mark_ended();
        tx.send(Message::Finished(summary))
            .context(BrokenPipeErr {})?;
    } else {
        scan(&ctx, on_message).await?;
    }

    ui_thread.join().unwrap()?;

    Ok(())
}

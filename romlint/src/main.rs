mod ansi;
mod args;
mod commands;
mod config;
mod db;
mod error;
mod filemeta;
mod linter;
mod scripts;
mod ui;
mod word_match;

use args::Args;
use clap::Parser;
use commands::{check, scan};
use error::{BrokenPipeErr, IoErr, Result};
use filemeta::FileMeta;
use scripts::{Requirements, ScriptLoader};
use snafu::ResultExt;
use std::sync::Arc;
use std::time::Instant;
use std::{sync::mpsc, thread::spawn};
use tokio::fs::read_dir;
use ui::{Message, Summary, Ui};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    if let Err(err) = run(args).await {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<()> {
    let config_path = args.config_path();
    let config = config::from_path(config_path).await?;
    let db_path = args.config_dir()?.join(config.db_dir());
    let (tx, rx) = mpsc::channel();
    let mut script_loader = ScriptLoader::new();

    if let Some(sys) = args.dump_system {
        let sys = sys.as_str();
        let database = db::load_only(&db_path, &[sys], &tx).await?;
        database.get(sys).unwrap().dump();
        return Ok(());
    }

    let mut dir = read_dir("./lints").await.unwrap();
    while let Ok(Some(file)) = dir.next_entry().await {
        script_loader.load(file.path()).await.unwrap();
    }

    let ui_thread = spawn(move || Ui::new(rx, !args.hide_passes).run());
    let databases = if let Some(sys) = &args.system {
        db::load_only(&db_path, &[sys.as_str()], &tx).await?
    } else {
        db::load_all(&db_path, &tx).await?
    };
    let databases = Arc::new(databases);

    if let Some(file) = args.file.as_ref() {
        let start_time = Instant::now();
        let mut summary = Summary::new(start_time);
        let read_archives = script_loader.requirements().contains(Requirements::ARCHIVE);
        let cwd = args.cwd();
        let system = args.system.as_deref();
        let file = FileMeta::from_path(system, &config, file, read_archives)
            .await
            .context(IoErr { path: file })?;
        let passed = check(cwd, &file, &script_loader, &tx, databases)?;

        if passed {
            summary.add_success(system.unwrap());
        } else {
            summary.add_failure(system.unwrap());
        }

        summary.mark_ended();
        tx.send(Message::Finished(summary))
            .context(BrokenPipeErr {})?;
    } else {
        scan(&args, &config, &script_loader, tx.clone(), databases).await?;
    }

    ui_thread.join().unwrap()?;

    Ok(())
}

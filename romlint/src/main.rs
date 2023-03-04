mod ansi;
mod args;
mod commands;
mod config;
mod db;
mod error;
mod filemeta;
mod linter;
mod rules;
mod ui;
mod word_match;

use args::Args;
use clap::Parser;
use commands::{check, scan};
use error::{BrokenPipeErr, IoErr, Result};
use filemeta::FileMeta;
use linter::Rules;
use rules::{
    ArchivedRomName, FilePermissions, MultifileArchive, NoLooseFiles, ObsoleteFormat,
    UncompressedFile, UnknownRom,
};
use snafu::ResultExt;
use std::time::Instant;
use std::{sync::mpsc, thread::spawn};
use ui::{Message, Summary, Ui};

#[tokio::main(flavor = "current_thread")]
async fn main() {
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

    if let Some(sys) = args.dump_system {
        let sys = sys.as_str();
        let database = db::load_only(&db_path, &[sys], &tx).await?;
        database.get(sys).unwrap().dump();
        return Ok(());
    }

    let ui_thread = spawn(move || Ui::new(rx).run());
    let databases = if let Some(sys) = &args.system {
        db::load_only(&db_path, &[sys.as_str()], &tx).await?
    } else {
        db::load_all(&db_path, &tx).await?
    };

    let rules: Rules = vec![
        Box::new(NoLooseFiles),
        Box::new(FilePermissions),
        Box::new(ObsoleteFormat),
        Box::new(UnknownRom::new(databases)),
        Box::new(UncompressedFile::default()),
        Box::new(MultifileArchive),
        Box::new(ArchivedRomName),
    ];

    if let Some(file) = args.file.as_ref() {
        let start_time = Instant::now();
        let mut summary = Summary::new(start_time);
        let read_archives = !args.no_archive_checks;
        let cwd = args.cwd();
        let system = args.system.as_ref().map(|s| s.as_str());
        let file = FileMeta::from_path(system, &config, file, read_archives)
            .await
            .context(IoErr { path: file })?;
        let passed = check(cwd, &file, &rules, &tx, read_archives)?;

        if passed {
            summary.add_success(system.unwrap());
        } else {
            summary.add_failure(system.unwrap());
        }

        summary.mark_ended();
        tx.send(Message::Finished(summary))
            .context(BrokenPipeErr {})?;
    } else {
        scan(&args, &config, rules, tx.clone()).await?;
    }

    ui_thread.join().unwrap()?;

    Ok(())
}

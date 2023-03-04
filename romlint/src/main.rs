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
use commands::scan;
use error::Result;
use linter::Rules;
use rules::{
    ArchivedRomName, FilePermissions, MultifileArchive, NoLooseFiles, ObsoleteFormat,
    UncompressedFile, UnknownRom,
};
use std::{sync::mpsc, thread::spawn};
use ui::Ui;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    if let Err(err) = run(args).await {
        eprintln!("{}", err);
    }
}

async fn run(args: Args) -> Result<()> {
    let config_path = args.config_path();
    let config = config::from_path(config_path).await?;
    let db_path = args.config_dir()?.join(config.db_dir());

    let (tx, rx) = mpsc::channel();
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

    scan(&args, &config, rules, tx.clone()).await?;

    ui_thread.join().unwrap()?;

    Ok(())
}

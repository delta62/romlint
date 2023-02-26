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
use db::Database;
use linter::Rules;
use rules::{
    FilePermissions, MultifileArchive, NoArchives, NoJunkFiles, ObsoleteFormat, UncompressedFile,
    UnknownRom,
};
use std::{sync::mpsc, thread::spawn};
use ui::{Message, Ui};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let config = config::from_path("./config.toml").await.unwrap();
    let db = Database::from_file(args.db.as_str()).await.unwrap();
    let (tx, rx) = mpsc::channel();
    let ui_thread = spawn(move || Ui::new(rx).run());

    let rules: Rules = vec![
        Box::new(NoJunkFiles),
        Box::new(NoArchives),
        Box::new(FilePermissions),
        Box::new(ObsoleteFormat),
        Box::new(UnknownRom::new(db)),
        Box::new(UncompressedFile),
        Box::new(MultifileArchive),
    ];

    scan(&args, &config, &rules, tx.clone()).await.unwrap();

    tx.send(Message::Finished).unwrap();
    ui_thread.join().unwrap();
}

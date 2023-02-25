mod ansi;
mod args;
mod db;
mod error;
mod filemeta;
mod linter;
mod rules;
mod ui;
mod word_match;

use args::Args;
use clap::Parser;
use db::Database;
use dir_walker::walk;
use filemeta::FileMeta;
use futures::TryStreamExt;
use linter::Rule;
use rules::{
    FilePermissions, MultifileArchive, NoArchives, NoJunkFiles, ObsoleteFormat, UncompressedFile,
    UnknownRom,
};
use std::{path::PathBuf, sync::mpsc};
use ui::{Message, Report, Ui};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let db = Database::from_file(args.db.as_str()).await.unwrap();
    let path = PathBuf::from(args.cwd.as_str());
    let (tx, rx) = mpsc::channel();
    let ui_thread = std::thread::spawn(move || Ui::new(rx).run());

    let mut stream = Box::pin(
        walk(path)
            .await
            .unwrap()
            .and_then(|file| async move { FileMeta::from_dir_walker(file).await }),
    );

    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(NoJunkFiles),
        Box::new(NoArchives),
        Box::new(FilePermissions),
        Box::new(ObsoleteFormat),
        Box::new(UnknownRom::new(db)),
        Box::new(UncompressedFile),
        Box::new(MultifileArchive),
    ];

    loop {
        let next = stream.try_next().await;
        match next {
            Ok(Some(file)) => {
                let filename = file
                    .path()
                    .strip_prefix(args.cwd.as_str())
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                tx.send(Message::SetStatus(filename.clone())).unwrap();

                let diagnostics: Vec<_> =
                    rules.iter().filter_map(|rule| rule.check(&file)).collect();

                let report = Report {
                    diagnostics,
                    path: filename,
                };

                tx.send(Message::Report(report)).unwrap();
            }
            Ok(None) => break,
            Err(err) => panic!("{}", err),
        }
    }

    tx.send(Message::Finished).unwrap();
    ui_thread.join().unwrap();
}

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

use args::{Args, Command};
use clap::Parser;
use commands::{dump, lint};
use error::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    let res = match args.command {
        Command::Dump => dump(args).await,
        Command::Lint(ref lint_args) => lint(&args, lint_args).await,
    };

    if let Err(err) = res {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

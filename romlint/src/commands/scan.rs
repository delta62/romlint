use super::check;
use crate::args::Args;
use crate::config::Config;
use crate::error::{BrokenPipeErr, IoErr};
use crate::filemeta::FileMeta;
use crate::linter::Rules;
use crate::ui::{Message, Summary};
use crate::Result;
use dir_walker::walk;
use futures::TryStreamExt;
use snafu::prelude::*;
use std::sync::mpsc::Sender;
use std::time::Instant;

pub async fn scan(args: &Args, config: &Config, rules: Rules, tx: Sender<Message>) -> Result<()> {
    let start_time = Instant::now();
    let mut summary = Summary::new(start_time);
    let cwd = args.cwd();
    let path = cwd.as_path();
    let system = args.system.clone();
    let system = system.as_deref();
    let read_archives = !args.no_archive_checks;

    let mut stream = Box::pin(walk(path).await.context(IoErr { path })?.and_then(
        |file| async move { FileMeta::from_dir_walker(file, system, config, read_archives).await },
    ));

    while let Some(file) = stream.try_next().await.context(IoErr { path })? {
        let system = file.system().unwrap_or("unknown");
        let pass = check(path, &file, &rules, &tx, read_archives)?;

        if pass {
            summary.add_success(system);
        } else {
            summary.add_failure(system);
        }
    }

    summary.mark_ended();

    tx.send(Message::Finished(summary))
        .context(BrokenPipeErr {})?;

    Ok(())
}

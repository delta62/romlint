use super::check;
use super::lint::LintContext;
use crate::error::IoErr;
use crate::filemeta::FileMeta;
use crate::ui::{Message, Summary};
use crate::Result;
use dir_walker::walk;
use futures::TryStreamExt;
use snafu::prelude::*;
use std::time::Instant;

pub async fn scan<F>(ctx: &LintContext, send: F) -> Result<()>
where
    F: Fn(Message) -> Result<()>,
{
    let mut summary = Summary::new(Instant::now());
    let read_archives = ctx.should_read_archives();
    let path = ctx.scan_dirs();

    let mut stream = Box::pin(walk(path).await.context(IoErr { path })?.and_then(
        |file| async move { FileMeta::from_dir_walker(file, system, config, read_archives).await },
    ));

    while let Some(file) = stream.try_next().await.context(IoErr { path })? {
        let system = file.system().unwrap_or("unknown");
        let pass = check(&ctx, &file, &send)?;

        if pass {
            summary.add_success(system);
        } else {
            summary.add_failure(system);
        }
    }

    summary.mark_ended();

    send(Message::Finished(summary))
}

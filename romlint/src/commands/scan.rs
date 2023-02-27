use super::check;
use crate::args::Args;
use crate::config::Config;
use crate::error::IoErr;
use crate::filemeta::FileMeta;
use crate::linter::Rules;
use crate::ui::Message;
use crate::Result;
use dir_walker::walk;
use futures::TryStreamExt;
use snafu::prelude::*;
use std::sync::mpsc::Sender;

pub async fn scan(args: &Args, config: &Config, rules: &Rules, tx: Sender<Message>) -> Result<()> {
    let cwd = args.cwd();
    let path = cwd.as_path();
    let system = args.system.clone();
    let system = system.as_deref();

    let mut stream = Box::pin(
        walk(path)
            .await
            .context(IoErr { path })?
            .and_then(|file| async move { FileMeta::from_dir_walker(file, system, config).await }),
    );

    loop {
        let next = stream.try_next().await.context(IoErr { path })?;
        match next {
            Some(file) => check(path, &file, rules, &tx)?,
            None => break,
        }
    }

    Ok(())
}

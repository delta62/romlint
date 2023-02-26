use super::check;
use crate::args::Args;
use crate::config::Config;
use crate::filemeta::FileMeta;
use crate::linter::Rules;
use crate::ui::Message;
use dir_walker::walk;
use futures::TryStreamExt;
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub async fn scan(
    args: &Args,
    config: &Config,
    rules: &Rules,
    tx: Sender<Message>,
) -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(args.cwd.as_str());
    let cwd = args.cwd.as_str();
    let mut stream = Box::pin(walk(path).await?.and_then(|file| async move {
        FileMeta::from_dir_walker(file, args.system.as_str(), config).await
    }));

    loop {
        let next = stream.try_next().await?;
        match next {
            Some(file) => check(cwd, &file, rules, &tx)?,
            None => break,
        }
    }

    Ok(())
}

use super::check;
use crate::args::Args;
use crate::config::Config;
use crate::error::Error;
use crate::filemeta::FileMeta;
use crate::linter::Rules;
use crate::ui::Message;
use crate::Result;
use dir_walker::walk;
use futures::TryStreamExt;
use std::sync::mpsc::Sender;

pub async fn scan(args: &Args, config: &Config, rules: &Rules, tx: Sender<Message>) -> Result<()> {
    let cwd = args.cwd();
    let path = cwd.as_path();

    let mut stream = Box::pin(walk(path.to_path_buf()).await.map_err(Error::Io)?.and_then(
        |file| async move { FileMeta::from_dir_walker(file, args.system.as_str(), config).await },
    ));

    loop {
        let next = stream.try_next().await.map_err(Error::Io)?;
        match next {
            Some(file) => check(path, &file, rules, &tx)?,
            None => break,
        }
    }

    Ok(())
}

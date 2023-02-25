use crate::args::Args;
use crate::filemeta::FileMeta;
use crate::linter::Rules;
use crate::ui::{Message, Report};
use dir_walker::walk;
use futures::TryStreamExt;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub async fn scan(args: &Args, rules: &Rules, tx: &mut Sender<Message>) {
    let path = PathBuf::from(args.cwd.as_str());
    let cwd = args.cwd.as_str();
    let mut stream = Box::pin(
        walk(path)
            .await
            .unwrap()
            .and_then(|file| async move { FileMeta::from_dir_walker(file).await }),
    );

    loop {
        let next = stream.try_next().await;
        match next {
            Ok(Some(file)) => {
                let filename = file
                    .path()
                    .strip_prefix(cwd)
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
}

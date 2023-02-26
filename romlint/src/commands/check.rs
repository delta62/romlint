use crate::linter::Rules;
use crate::{
    error::{BrokenPipeErr, Result},
    filemeta::FileMeta,
    ui::{Message, Report},
};
use snafu::prelude::*;
use std::path::Path;
use std::sync::mpsc::Sender;

pub fn check<P: AsRef<Path>>(
    cwd: P,
    file: &FileMeta,
    rules: &Rules,
    tx: &Sender<Message>,
) -> Result<()> {
    let path = file
        .path()
        .strip_prefix(cwd)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    tx.send(Message::SetStatus(path.clone()))
        .context(BrokenPipeErr {})?;

    let diagnostics = rules
        .iter()
        .filter_map(|rule| rule.check(file))
        .collect::<Vec<_>>();
    let report = Report { diagnostics, path };

    tx.send(Message::Report(report)).context(BrokenPipeErr {})?;

    Ok(())
}

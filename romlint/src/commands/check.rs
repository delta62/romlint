use crate::linter::Rules;
use crate::{
    error::Result,
    filemeta::FileMeta,
    ui::{Message, Report},
};
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
        .to_string_lossy()
        .to_string();

    tx.send(Message::SetStatus(path.clone())).unwrap();

    let diagnostics = rules
        .iter()
        .filter_map(|rule| rule.check(file))
        .collect::<Vec<_>>();
    let report = Report { diagnostics, path };

    tx.send(Message::Report(report)).unwrap();

    Ok(())
}

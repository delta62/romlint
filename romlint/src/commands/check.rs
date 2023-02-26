use crate::linter::Rules;
use crate::{
    filemeta::FileMeta,
    ui::{Message, Report},
};
use std::error::Error;
use std::path::Path;
use std::sync::mpsc::Sender;

pub fn check<P: AsRef<Path>>(
    cwd: P,
    file: &FileMeta,
    rules: &Rules,
    tx: &Sender<Message>,
) -> Result<(), Box<dyn Error>> {
    let path = file
        .path()
        .strip_prefix(cwd)
        .unwrap()
        .to_string_lossy()
        .to_string();

    tx.send(Message::SetStatus(path.clone()))?;

    let diagnostics: Vec<_> = rules.iter().filter_map(|rule| rule.check(file)).collect();
    let report = Report { diagnostics, path };

    tx.send(Message::Report(report))?;

    Ok(())
}

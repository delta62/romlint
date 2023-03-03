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
    rules: &mut Rules,
    tx: &Sender<Message>,
) -> Result<bool> {
    let path = file
        .path()
        .strip_prefix(cwd)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    tx.send(Message::SetStatus(path.clone()))
        .context(BrokenPipeErr {})?;

    let mut diagnostics = Vec::new();

    for rule in rules {
        if let Err(diag) = rule.check(file) {
            let terminal = diag.terminal;

            diagnostics.push(diag);

            if terminal {
                break;
            }
        }
    }

    let passed = diagnostics.is_empty();
    let report = Report { diagnostics, path };

    tx.send(Message::Report(report)).context(BrokenPipeErr {})?;

    Ok(passed)
}

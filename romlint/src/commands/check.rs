use crate::linter::Diagnostic;
use crate::scripts::{exec_one, ScriptLoader};
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
    file: &FileMeta<'_>,
    script_loader: &ScriptLoader,
    tx: &Sender<Message>,
) -> Result<bool> {
    use rlua::Error::*;

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

    for lint in script_loader.iter() {
        if let Err(err) = exec_one(lint, file) {
            let message;

            match err {
                CallbackError { traceback, cause } => {
                    message = format!("{cause} - {traceback}");
                }
                err => {
                    message = format!("{err}");
                }
            }

            let diag = Diagnostic::from_file(file, message);
            diagnostics.push(diag);
        }
    }

    let passed = diagnostics.is_empty();
    let report = Report { diagnostics, path };

    tx.send(Message::Report(report)).context(BrokenPipeErr {})?;

    Ok(passed)
}

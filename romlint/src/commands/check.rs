use crate::db::Database;
use crate::linter::Diagnostic;
use crate::scripts::{exec_one, ScriptLoader};
use crate::{
    error::{BrokenPipeErr, Result},
    filemeta::FileMeta,
    ui::{Message, Report},
};
use snafu::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::Arc;

pub fn check<P: AsRef<Path>>(
    cwd: P,
    file: &FileMeta<'_>,
    script_loader: &ScriptLoader,
    tx: &Sender<Message>,
    databases: Arc<HashMap<String, Database>>,
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
        if let Err(err) = exec_one(lint, file, databases.clone()) {
            let message = match err {
                CallbackError { cause, .. } => format!("{cause}"),
                err => format!("{err}"),
            };

            let diag = Diagnostic::from_file(file, message);
            diagnostics.push(diag);
        }
    }

    let passed = diagnostics.is_empty();
    let report = Report { diagnostics, path };

    tx.send(Message::Report(report)).context(BrokenPipeErr {})?;

    Ok(passed)
}

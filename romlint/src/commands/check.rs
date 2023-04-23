use crate::linter::Rules;
use crate::scripts::ScriptHost;
use crate::{
    error::{BrokenPipeErr, Result},
    filemeta::FileMeta,
    ui::{Message, Report},
};
use snafu::prelude::*;
use std::path::Path;
use std::sync::mpsc::Sender;

pub async fn check<P: AsRef<Path>>(
    cwd: P,
    file: &FileMeta<'_>,
    rules: &Rules,
    tx: &Sender<Message>,
    read_archives: bool,
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
    let mut script_host = ScriptHost::new();

    script_host.load("lints/file_mode.lua").await.unwrap();
    if let Err(err) = script_host.exec_all(file) {
        match err {
            rlua::Error::CallbackError { traceback, cause } => {
                log::error!("{cause} - {traceback}");
            }
            err => log::error!("{err}"),
        }
    }

    // for rule in rules {
    //     if let Err(diag) = rule.check(file) {
    //         let terminal = diag.terminal;

    //         diagnostics.push(diag);

    //         if terminal {
    //             break;
    //         }
    //     }

    //     if read_archives {
    //         if let Err(diag) = rule.check_archive(file) {
    //             let terminal = diag.terminal;

    //             diagnostics.push(diag);

    //             if terminal {
    //                 break;
    //             }
    //         }
    //     }
    // }

    let passed = diagnostics.is_empty();
    let report = Report { diagnostics, path };

    tx.send(Message::Report(report)).context(BrokenPipeErr {})?;

    Ok(passed)
}

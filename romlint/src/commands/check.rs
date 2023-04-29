use super::lint::LintContext;
use crate::{
    error::{InvalidPathErr, Result},
    filemeta::FileMeta,
    linter::Diagnostic,
    scripts::exec_one,
    ui::{Message, Report},
};
use snafu::OptionExt;

pub fn check<F>(ctx: &LintContext, file: &FileMeta<'_>, send: F) -> Result<bool>
where
    F: Fn(Message) -> Result<()>,
{
    let path = ctx
        .relative_path(file.path())
        .and_then(|p| p.to_str().map(|s| s.to_owned()))
        .context(InvalidPathErr { path: file.path() })?;

    send(Message::SetStatus(path.clone()))?;

    let diagnostics = ctx.scripts().fold(Vec::new(), |mut acc, lint| {
        let result = exec_one(lint, file, ctx.databases());
        if let Err(err) = result {
            let diag = create_diagnostic(file, err);
            acc.push(diag);
        }

        acc
    });

    let passed = diagnostics.is_empty();
    let report = Report { diagnostics, path };
    send(Message::Report(report))?;

    Ok(passed)
}

fn create_diagnostic(file: &FileMeta, err: rlua::Error) -> Diagnostic {
    use rlua::Error::*;

    let message = match err {
        CallbackError { cause, .. } => format!("{cause}"),
        err => format!("{err}"),
    };

    Diagnostic::from_file(file, message)
}

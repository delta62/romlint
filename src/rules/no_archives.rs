use crate::dir_walker::FileMeta;
use crate::linter::{Diagnostic, Rule};

const ARCHIVE_EXTENSIONS: [&str; 1] = ["7z"];

pub struct NoArchives;

impl Rule for NoArchives {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let path = file.entry.path();
        let ext = path.extension();

        ARCHIVE_EXTENSIONS
            .iter()
            .find(|&e| e == &ext.and_then(|e| e.to_str()).unwrap_or(""))
            .map(|extension| Diagnostic {
                path: file.entry.path(),
                message: format!("Unextracted archive ({})", extension),
                hints: vec![],
            })
    }
}

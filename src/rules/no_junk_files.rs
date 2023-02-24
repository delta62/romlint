use crate::dir_walker::FileMeta;
use crate::linter::{Diagnostic, Rule};
use std::path::Path;

const JUNK_FILES: [&str; 1] = ["txt"];

pub struct NoJunkFiles;

impl Rule for NoJunkFiles {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let filename = file.entry.file_name();
        let path = Path::new(&filename);

        JUNK_FILES
            .iter()
            .find(|&e| e == &path.extension().and_then(|e| e.to_str()).unwrap_or(""))
            .map(|extension| Diagnostic {
                path: file.entry.path(),
                message: format!("Junk file extension (.{})", extension),
                hints: vec![],
            })
    }
}

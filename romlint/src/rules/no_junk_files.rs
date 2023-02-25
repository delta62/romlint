use crate::linter::{Diagnostic, Rule};
use dir_walker::FileMeta;

const JUNK_FILES: [&str; 1] = ["txt"];

pub struct NoJunkFiles;

impl Rule for NoJunkFiles {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let extension = file.extension().unwrap_or("");

        JUNK_FILES
            .iter()
            .find(|&e| e == &extension)
            .map(|extension| Diagnostic {
                path: file.path().to_path_buf(),
                message: format!("Junk file extension (.{})", extension),
                hints: vec![],
            })
    }
}

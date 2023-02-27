use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

const JUNK_FILES: [&str; 1] = ["txt"];

pub struct NoJunkFiles;

impl Rule for NoJunkFiles {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let extension = file.extension().unwrap_or("");

        JUNK_FILES
            .iter()
            .find(|&e| e == &extension)
            .map(|extension| {
                Diagnostic::from_file(file, format!("Junk file extension (.{})", extension))
            })
    }
}

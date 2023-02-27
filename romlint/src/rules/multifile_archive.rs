use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct MultifileArchive;

impl Rule for MultifileArchive {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        // For files which are not archived, this rule is not applicable
        let len = file.archive().map(|arc| arc.len()).unwrap_or(1);

        if len == 1 {
            None
        } else if len == 0 {
            Some(Diagnostic::from_file(file, "archive is empty"))
        } else {
            Some(Diagnostic::from_file(
                file,
                "archive should have exactly 1 item",
            ))
        }
    }
}

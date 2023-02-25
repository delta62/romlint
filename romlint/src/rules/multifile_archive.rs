use crate::linter::{Diagnostic, Rule};
use dir_walker::FileMeta;

pub struct MulitfileArchive;

impl Rule for MulitfileArchive {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        // For files which are not archived, this rule is not applicable
        let len = file.archive().map(|arc| arc.len()).unwrap_or(1);

        if len == 1 {
            None
        } else if len == 0 {
            Some(Diagnostic {
                message: "archive is empty".to_owned(),
                path: file.path().to_owned(),
                hints: vec![],
            })
        } else {
            Some(Diagnostic {
                message: "archive should have exactly 1 item".to_owned(),
                path: file.path().to_owned(),
                hints: vec![],
            })
        }
    }
}

use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct MultifileArchive;

impl Rule for MultifileArchive {
    fn check(&self, _file: &FileMeta) -> Result<(), Diagnostic> {
        Ok(())
    }

    fn check_archive(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        // For files which are not archived, this rule is not applicable
        let len = file.archive().map(|arc| arc.len()).unwrap_or(1);

        if len == 0 {
            Err(Diagnostic::from_file(file, "archive is empty"))?;
        }

        if len > 1 {
            Err(Diagnostic::from_file(
                file,
                "archive should have exactly 1 item",
            ))?
        }

        Ok(())
    }
}

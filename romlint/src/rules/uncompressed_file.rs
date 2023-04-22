use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

#[derive(Default)]
pub struct UncompressedFile;

impl Rule for UncompressedFile {
    fn check(&self, _file: &FileMeta) -> Result<(), Diagnostic> {
        Ok(())
    }

    fn check_archive(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        if !file.is_archived() {
            Err(Diagnostic::from_file(file, "File is not compressed"))?;
        }

        Ok(())
    }
}

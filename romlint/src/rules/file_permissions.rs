use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};
use std::os::unix::prelude::PermissionsExt;

pub struct FilePermissions;

impl Rule for FilePermissions {
    fn check(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        let mode = file.metadata().permissions().mode() & 0o777;
        let is_dir = file.metadata().is_dir();

        match (is_dir, mode) {
            (true, 0o755) | (false, 0o644) => Ok(()),
            (true, _) => Err(Diagnostic::from_file(
                file,
                format!(
                    "Directory has incorrect permissions; should be 755 (is {:o})",
                    mode
                ),
            )),
            (false, _) => Err(Diagnostic::from_file(
                file,
                format!(
                    "File has incorrect permissions; should be 644 (is {:o})",
                    mode
                ),
            )),
        }
    }

    fn check_archive(&self, _file: &FileMeta) -> Result<(), Diagnostic> {
        // TODO: Add permission metadata to compressed files
        Ok(())
    }
}

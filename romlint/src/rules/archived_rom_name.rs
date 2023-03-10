use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct ArchivedRomName;

impl Rule for ArchivedRomName {
    fn check(&self, _file: &FileMeta) -> Result<(), Diagnostic> {
        Ok(())
    }

    fn check_archive(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        // For files which are not archived, this rule is not applicable
        if let Some(archive) = file.archive() {
            let archive_basename = file.basename().unwrap();
            let contains_match = archive.file_names().any(|name| {
                name.file_stem()
                    .map(|stem| stem == archive_basename)
                    .unwrap_or(false)
            });

            if contains_match {
                return Ok(());
            }
        } else {
            return Ok(());
        }

        Err(Diagnostic::from_file(
            file,
            "archived file name should match the archive's name",
        ))
    }
}

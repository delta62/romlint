use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct ArchivedRomName;

impl Rule for ArchivedRomName {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        // For files which are not archived, this rule is not applicable
        if let Some(archive) = file.archive() {
            let archive_basename = file.basename().unwrap();
            let contains_match = archive.file_names().any(|name| name == archive_basename);

            if contains_match {
                return None;
            }
        } else {
            return None;
        }

        Some(Diagnostic {
            message: "archived file name should match the archive's name".to_owned(),
            path: file.path().to_owned(),
            hints: vec![],
        })
    }
}

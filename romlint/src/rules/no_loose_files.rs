use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};
use std::iter::once;

pub struct NoLooseFiles;

impl Rule for NoLooseFiles {
    fn check(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        let extension = file.extension().unwrap_or("");
        let config = file
            .config()
            .ok_or_else(|| Diagnostic::unknown_system(file))?;

        let mut allowed_extensions = config
            .archive_formats
            .iter()
            .chain(config.obsolete_formats.iter())
            .chain(once(&config.archive_format));

        let is_loose_file = allowed_extensions.any(|e| e == &extension);

        if is_loose_file {
            Err(Diagnostic::from_file(file, "Loose file"))?;
        }

        Ok(())
    }
}

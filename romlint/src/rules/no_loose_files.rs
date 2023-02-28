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

        let default_obs = vec![];
        let obsolete_formats = config.obsolete_formats.as_ref().unwrap_or(&default_obs);

        // Don't double-report things that other rules will report
        let mut allowed_extensions = config
            .archive_formats
            .iter()
            .chain(obsolete_formats.iter())
            .chain(once(&config.archive_format))
            .chain(once(&config.raw_format));

        let is_loose_file = !allowed_extensions.any(|e| e == &extension);

        if is_loose_file {
            Err(Diagnostic::from_file(file, "Loose file"))?;
        }

        Ok(())
    }
}

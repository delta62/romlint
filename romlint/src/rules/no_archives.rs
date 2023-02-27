use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct NoArchives;

impl Rule for NoArchives {
    fn check(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        let config = file
            .config()
            .ok_or_else(|| Diagnostic::unknown_system(file))?;

        let extensions = &config.archive_formats;
        let extension = file.extension().unwrap_or("");
        let found_ext = extensions.iter().any(|&e| e == extension);

        if found_ext {
            Err(Diagnostic::from_file(
                file,
                format!("Unextracted archive ({})", extension),
            ))?;
        }

        Ok(())
    }
}

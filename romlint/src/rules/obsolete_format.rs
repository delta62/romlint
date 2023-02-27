use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct ObsoleteFormat;

impl Rule for ObsoleteFormat {
    fn check(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        let config = file
            .config()
            .ok_or_else(|| Diagnostic::unknown_system(file))?;

        let obsolete_formats = &config.obsolete_formats;
        let extension = file.extension().unwrap_or("");
        let found_format = obsolete_formats.iter().any(|e| e == &extension);

        if found_format {
            Err(Diagnostic::from_file(
                file,
                format!("Obsolete format ({})", extension),
            ))?;
        }

        Ok(())
    }
}

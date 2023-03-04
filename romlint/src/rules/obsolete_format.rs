use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

const NO_FORMATS: Vec<&str> = Vec::new();

pub struct ObsoleteFormat;

impl Rule for ObsoleteFormat {
    fn check(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        let no_formats = NO_FORMATS;
        let obsolete_formats = file
            .config()
            .ok_or_else(|| Diagnostic::unknown_system(file))?
            .obsolete_formats
            .as_ref()
            .unwrap_or(&no_formats);

        let extension = file.extension().unwrap_or("");
        let found_format = obsolete_formats.iter().find(|&&e| e == extension).is_some();

        if found_format {
            Err(Diagnostic::from_file(
                file,
                format!("Obsolete format ({})", extension),
            ))?;
        }

        Ok(())
    }

    fn check_archive(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        if let Some(archive) = file.archive() {
            let no_formats = NO_FORMATS;
            let obsolete_formats = file
                .config()
                .ok_or_else(|| Diagnostic::unknown_system(file))?
                .obsolete_formats
                .as_ref()
                .unwrap_or(&no_formats);

            let mut extensions = archive
                .file_names()
                .filter_map(|name| name.extension())
                .filter_map(|name| name.to_str());

            let found_format = obsolete_formats
                .iter()
                .find(|obs| extensions.any(|ext| &ext == *obs));

            if let Some(obsolete) = found_format {
                Err(Diagnostic::from_file(
                    file,
                    format!("Obsolete format ({})", obsolete),
                ))?;
            }
        }

        Ok(())
    }
}

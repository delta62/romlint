use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

const NO_FORMATS: Vec<&str> = Vec::new();

pub struct NoLooseFiles;

impl Rule for NoLooseFiles {
    fn check(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        let extension = file.extension().unwrap_or("");
        let config = file
            .config()
            .ok_or_else(|| Diagnostic::unknown_system(file))?;

        let no_formats = NO_FORMATS;
        let obsolete_formats = config.obsolete_formats.as_ref().unwrap_or(&no_formats);

        // Don't double-report things that other rules will report
        let mut allowed_extensions = config
            .raw_format
            .iter()
            .chain(obsolete_formats.iter())
            .chain(config.archive_format.iter());

        let is_loose_file = !allowed_extensions.any(|e| e == &extension);

        if is_loose_file {
            Err(Diagnostic::from_file(file, "Loose file"))?;
        }

        Ok(())
    }

    fn check_archive(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        if let Some(archive) = file.archive() {
            let config = file
                .config()
                .ok_or_else(|| Diagnostic::unknown_system(file))?;

            let no_formats = &NO_FORMATS;
            let obsolete_formats = config.obsolete_formats.as_ref().unwrap_or(&no_formats);

            // Don't double-report things that other rules will report
            let mut allowed_extensions = config
                .raw_format
                .iter()
                .chain(obsolete_formats.iter())
                .chain(config.archive_format.iter());

            let loose_files = archive
                .file_names()
                .filter_map(|file| {
                    file.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| (ext, file))
                })
                .filter(|(ext, _file)| !allowed_extensions.any(|e| e == ext))
                .map(|(_ext, file)| {
                    format!("loose file in archive: {}", file.to_str().unwrap_or(""))
                })
                .collect::<Vec<_>>();

            if !loose_files.is_empty() {
                let err = Diagnostic::from_file(file, "Loose file").with_hints(loose_files);
                Err(err)?;
            }
        }

        Ok(())
    }
}

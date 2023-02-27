use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct NoArchives;

impl Rule for NoArchives {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let config = file.config();

        if let Some(conf) = config {
            let extensions = &conf.archive_formats;
            let extension = file.extension().unwrap_or("");

            extensions
                .iter()
                .find(|&&e| e == extension)
                .map(|extension| {
                    Diagnostic::from_file(file, format!("Unextracted archive ({})", extension))
                })
        } else {
            Some(Diagnostic::unknown_system(file))
        }
    }
}

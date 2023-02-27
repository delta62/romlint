use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct ObsoleteFormat;

impl Rule for ObsoleteFormat {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let config = file.config();

        if let Some(conf) = config {
            let obsolete_formats = &conf.obsolete_formats;
            let extension = file.extension().unwrap_or("");

            obsolete_formats
                .iter()
                .find(|&e| e == &extension)
                .map(|extension| {
                    Diagnostic::from_file(file, format!("Obsolete format ({})", extension))
                })
        } else {
            Some(Diagnostic::unknown_system(file))
        }
    }
}

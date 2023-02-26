use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct ObsoleteFormat;

impl Rule for ObsoleteFormat {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let obsolete_formats = &file.config().obsolete_formats;
        let extension = file.extension().unwrap_or("");

        obsolete_formats
            .iter()
            .find(|&e| e == &extension)
            .map(|extension| Diagnostic {
                path: file.path().to_path_buf(),
                message: format!("Obsolete format ({})", extension),
                hints: vec![],
            })
    }
}

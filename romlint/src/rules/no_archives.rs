use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct NoArchives;

impl Rule for NoArchives {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let extensions = &file.config().archive_formats;
        let extension = file.extension().unwrap_or("");

        extensions
            .iter()
            .find(|&&e| e == extension)
            .map(|extension| Diagnostic {
                path: file.path().to_path_buf(),
                message: format!("Unextracted archive ({})", extension),
                hints: vec![],
            })
    }
}

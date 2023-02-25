use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

const ARCHIVE_EXTENSIONS: [&str; 1] = ["7z"];

pub struct NoArchives;

impl Rule for NoArchives {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let extension = file
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        ARCHIVE_EXTENSIONS
            .iter()
            .find(|&&e| e == extension)
            .map(|extension| Diagnostic {
                path: file.path().to_path_buf(),
                message: format!("Unextracted archive ({})", extension),
                hints: vec![],
            })
    }
}

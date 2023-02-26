use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct UncompressedFile;

impl Rule for UncompressedFile {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let extension = file.extension().unwrap_or("");
        let archive_extension = &file.config().archive_format;
        let is_compressed = archive_extension == &extension;

        if is_compressed {
            None
        } else {
            Some(Diagnostic {
                path: file.path().to_path_buf(),
                message: "File is not compressed".to_string(),
                hints: vec![],
            })
        }
    }
}

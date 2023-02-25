use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

const COMPRESSED_FORMATS: [&str; 1] = ["zip"];

pub struct UncompressedFile;

impl Rule for UncompressedFile {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let extension = file.extension().unwrap_or("");
        let is_compressed = COMPRESSED_FORMATS.iter().any(|&e| e == extension);

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

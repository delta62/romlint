use crate::linter::{Diagnostic, Rule};
use dir_walker::FileMeta;

const COMPRESSED_FORMATS: [&str; 1] = ["zip"];

pub struct UncompressedFile;

impl Rule for UncompressedFile {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let extension = file.extension().unwrap_or("");

        let is_compressed = COMPRESSED_FORMATS
            .iter()
            .find(|&e| e == &extension)
            .is_some();

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

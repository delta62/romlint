use crate::dir_walker::FileMeta;
use crate::linter::{Diagnostic, Rule};

const COMPRESSED_FORMATS: [&str; 1] = ["zip"];

pub struct UncompressedFile;

impl Rule for UncompressedFile {
    fn check(&self, entry: &FileMeta) -> Option<Diagnostic> {
        let path = entry.entry.path();
        let ext = path.extension();

        let is_compressed = COMPRESSED_FORMATS
            .iter()
            .find(|&e| e == &ext.and_then(|e| e.to_str()).unwrap_or(""))
            .is_some();

        if is_compressed {
            None
        } else {
            Some(Diagnostic {
                path: entry.entry.path(),
                message: "File is not compressed".to_string(),
                hints: vec![],
            })
        }
    }
}

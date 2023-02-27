use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct UncompressedFile;

impl Rule for UncompressedFile {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let extension = file.extension().unwrap_or("");
        let config = file.config();

        if let Some(conf) = config {
            let archive_extension = &conf.archive_format;
            let is_compressed = archive_extension == &extension;

            if is_compressed {
                None
            } else {
                Some(Diagnostic::from_file(file, "File is not compressed"))
            }
        } else {
            Some(Diagnostic::unknown_system(file))
        }
    }
}

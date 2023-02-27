use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

pub struct UncompressedFile;

impl Rule for UncompressedFile {
    fn check(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        let extension = file.extension().unwrap_or("");
        let config = file
            .config()
            .ok_or_else(|| Diagnostic::unknown_system(file))?;

        let archive_extension = &config.archive_format;
        let is_compressed = archive_extension == &extension;

        if is_compressed {
            Ok(())
        } else {
            Err(Diagnostic::from_file(file, "File is not compressed"))
        }
    }
}

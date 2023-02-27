use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

const JUNK_FILES: [&str; 1] = ["txt"];

pub struct NoJunkFiles;

impl Rule for NoJunkFiles {
    fn check(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        let extension = file.extension().unwrap_or("");
        let found_extension = JUNK_FILES.iter().any(|e| e == &extension);

        if found_extension {
            Err(Diagnostic::from_file(
                file,
                format!("Junk file extension (.{})", extension),
            ))?;
        }

        Ok(())
    }
}

use crate::dir_walker::FileMeta;
use crate::linter::{Diagnostic, Rule};

const OBSOLETE_FORMATS: [&str; 2] = ["n64", "v64"];

pub struct ObsoleteFormat;

impl Rule for ObsoleteFormat {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        let extension = file.extension().unwrap_or("");

        OBSOLETE_FORMATS
            .iter()
            .find(|&e| e == &extension)
            .map(|extension| Diagnostic {
                path: file.path().to_path_buf(),
                message: format!("Obsolete format ({})", extension),
                hints: vec![],
            })
    }
}

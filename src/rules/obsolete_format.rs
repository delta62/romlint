use crate::dir_walker::FileMeta;
use crate::linter::{Diagnostic, Rule};

const OBSOLETE_FORMATS: [&'static str; 2] = ["n64", "v64"];

pub struct ObsoleteFormat;

impl Rule for ObsoleteFormat {
    fn check(&self, entry: &FileMeta) -> Option<Diagnostic> {
        let path = entry.entry.path();
        let ext = path.extension();

        OBSOLETE_FORMATS
            .iter()
            .find(|&e| e == &ext.and_then(|e| e.to_str()).unwrap_or(""))
            .map(|extension| Diagnostic {
                path: entry.entry.path(),
                message: format!("Obsolete format ({})", extension),
                hints: vec![],
            })
    }
}

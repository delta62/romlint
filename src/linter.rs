use std::path::{Path, PathBuf};

use crate::dir_walker::FileMeta;

const JUNK_FILES: [&'static str; 1] = ["txt"];
const ARCHIVE_EXTENSIONS: [&'static str; 1] = ["7z"];

pub struct Diagnostic {
    pub message: String,
    pub path: PathBuf,
}

pub trait Rule {
    fn check(&self, entry: &FileMeta) -> Option<Diagnostic>;
}

pub struct NoJunkFiles;

impl Rule for NoJunkFiles {
    fn check(&self, entry: &FileMeta) -> Option<Diagnostic> {
        let filename = entry.entry.file_name();
        let path = Path::new(&filename);

        JUNK_FILES
            .iter()
            .find(|&e| e == &path.extension().and_then(|e| e.to_str()).unwrap_or(""))
            .map(|extension| Diagnostic {
                path: entry.entry.path(),
                message: format!("Junk file extension [{}]", extension),
            })
    }
}

pub struct NoArchives;

impl Rule for NoArchives {
    fn check(&self, entry: &FileMeta) -> Option<Diagnostic> {
        let path = entry.entry.path();
        let ext = path.extension();

        ARCHIVE_EXTENSIONS
            .iter()
            .find(|&e| e == &ext.and_then(|e| e.to_str()).unwrap_or(""))
            .map(|extension| Diagnostic {
                path: entry.entry.path(),
                message: format!("Unextracted archive [{}]", extension),
            })
    }
}

use std::{
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::{db::DataFile, dir_walker::FileMeta};

const JUNK_FILES: [&'static str; 1] = ["txt"];
const ARCHIVE_EXTENSIONS: [&'static str; 1] = ["7z"];
const OBSOLETE_FORMATS: [&'static str; 2] = ["n64", "v64"];

#[derive(Debug)]
pub struct Diagnostic {
    pub message: String,
    pub path: PathBuf,
    pub hints: Vec<String>,
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
                message: format!("Junk file extension (.{})", extension),
                hints: vec![],
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
                message: format!("Unextracted archive ({})", extension),
                hints: vec![],
            })
    }
}

pub struct FilePermissions;

impl Rule for FilePermissions {
    fn check(&self, entry: &FileMeta) -> Option<Diagnostic> {
        let mode = entry.meta.permissions().mode() & 0o777;
        let is_dir = entry.meta.is_dir();

        match (is_dir, mode) {
            (true, 0o755) | (false, 0o644) => None,
            (true, _) => Some(Diagnostic {
                message: format!(
                    "Directory has incorrect permissions; should be 755 (is {:o})",
                    mode
                ),
                path: entry.entry.path(),
                hints: vec![],
            }),
            (false, _) => Some(Diagnostic {
                message: format!(
                    "File has incorrect permissions; should be 644 (is {:o})",
                    mode
                ),
                path: entry.entry.path(),
                hints: vec![],
            }),
        }
    }
}

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

pub struct UnknownRom {
    datafile: DataFile,
}

impl UnknownRom {
    pub fn new(datafile: DataFile) -> Self {
        Self { datafile }
    }
}

impl Rule for UnknownRom {
    fn check(&self, entry: &FileMeta) -> Option<Diagnostic> {
        if self
            .datafile
            .contains(entry.entry.file_name().into_string().unwrap().as_str())
        {
            None
        } else {
            let path_str = entry.entry.path();
            let path_str = path_str.to_str().unwrap();
            let file_tokens = Tokens::from_str(path_str);
            let similar_titles = self.datafile.similar_to(&file_tokens);
            let mut hints = similar_titles
                .map(|title| title.to_string())
                .collect::<Vec<_>>();

            if !hints.is_empty() {
                hints.insert(0, "Some similar titles were found:".to_string())
            }

            Some(Diagnostic {
                message: "Can't find this ROM in the database".to_string(),
                path: entry.entry.path(),
                hints,
            })
        }
    }
}

pub struct Tokens<'a> {
    tags: Vec<&'a str>,
    words: Vec<&'a str>,
}

impl<'a> Tokens<'a> {
    pub fn from_str(s: &'a str) -> Self {
        let tokens = match s.rsplit_once('.') {
            Some((name, _ext)) => name,
            None => s,
        };
        let tokens = tokens.split_whitespace();
        let (tags, words) = tokens.partition(|&token| {
            token.starts_with('(') && token.ends_with(')')
                || token.starts_with('[') && token.ends_with(']')
        });

        Self { tags, words }
    }

    pub fn words_in_common_with(&self, other: &Tokens) -> usize {
        self.words
            .iter()
            .filter(|word| other.words.contains(word))
            .count()
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }
}

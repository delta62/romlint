use crate::filemeta::FileMeta;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub path: PathBuf,
    pub hints: Option<Vec<String>>,
    pub terminal: bool,
}

impl Diagnostic {
    pub fn from_file<M: Into<String>>(file: &FileMeta, message: M) -> Self {
        Self {
            hints: None,
            message: message.into(),
            path: file.path().to_path_buf(),
            terminal: false,
        }
    }
}

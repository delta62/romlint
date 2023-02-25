use crate::dir_walker::FileMeta;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Diagnostic {
    pub message: String,
    pub path: PathBuf,
    pub hints: Vec<String>,
}

pub trait Rule {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic>;
}

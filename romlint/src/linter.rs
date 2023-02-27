use crate::filemeta::FileMeta;
use std::path::PathBuf;

#[derive(Debug)]
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

    pub fn with_hints(mut self, hints: Vec<String>) -> Self {
        self.hints = Some(hints);
        self
    }

    pub fn unknown_system(file: &FileMeta) -> Self {
        let system_name = file.system().unwrap_or("none");

        Self {
            message: "Can't find configuration data for this system".to_owned(),
            path: file.path().to_path_buf(),
            hints: Some(vec![format!("Detected system: {}", system_name)]),
            terminal: true,
        }
    }
}

pub trait Rule {
    fn check(&self, file: &FileMeta) -> Result<(), Diagnostic>;
}

pub type Rules = Vec<Box<dyn Rule + Sync + Send>>;

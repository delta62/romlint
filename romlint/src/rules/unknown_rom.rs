use crate::db::Database;
use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};
use crate::word_match::Tokens;
use std::collections::HashMap;

pub struct UnknownRom {
    databases: HashMap<String, Database>,
}

impl UnknownRom {
    pub fn new(databases: HashMap<String, Database>) -> Self {
        Self { databases }
    }
}

impl Rule for UnknownRom {
    fn check(&self, file: &FileMeta) -> Result<(), Diagnostic> {
        let db = file
            .system()
            .and_then(|sys| self.databases.get(sys))
            .ok_or_else(|| Diagnostic::from_file(file, "Can't find a ROM database to search"))?;

        let file_name = file.path().file_stem().unwrap().to_str().unwrap();
        if db.contains(file_name) {
            return Ok(());
        }

        let path_str = file.path().to_str().unwrap();
        let file_tokens = Tokens::from_str(path_str);
        let similar_titles = db.similar_to(&file_tokens);
        let mut hints = similar_titles
            .iter()
            .map(|title| format!("* {}", title))
            .collect::<Vec<_>>();

        if !hints.is_empty() {
            hints.insert(0, "Some similar titles were found:".to_string())
        }

        Err(Diagnostic::from_file(file, "Can't find this ROM in the database").with_hints(hints))
    }
}

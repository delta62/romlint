use crate::db::DataFile;
use crate::dir_walker::FileMeta;
use crate::linter::{Diagnostic, Rule};
use crate::word_match::Tokens;

pub struct UnknownRom {
    datafile: DataFile,
}

impl UnknownRom {
    pub fn new(datafile: DataFile) -> Self {
        Self { datafile }
    }
}

impl Rule for UnknownRom {
    fn check(&self, file: &FileMeta) -> Option<Diagnostic> {
        if self
            .datafile
            .contains(file.path().file_name().unwrap().to_str().unwrap())
        {
            None
        } else {
            let path_str = file.path().to_str().unwrap();
            let file_tokens = Tokens::from_str(path_str);
            let similar_titles = self.datafile.similar_to(&file_tokens);
            let mut hints = similar_titles
                .iter()
                .map(|title| format!("* {}", title))
                .collect::<Vec<_>>();

            if !hints.is_empty() {
                hints.insert(0, "Some similar titles were found:".to_string())
            }

            Some(Diagnostic {
                message: "Can't find this ROM in the database".to_string(),
                path: file.path().to_path_buf(),
                hints,
            })
        }
    }
}

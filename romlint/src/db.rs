use crate::error::{DatabaseNameErr, DatabaseReadErr, IoErr, Result};
use crate::word_match::Tokens;
use crate::{args::Args, config::Config};
use no_intro::DataFile;
use snafu::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::{read_dir, read_to_string};

pub struct Database(DataFile);

impl Database {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let s = read_to_string(path.as_ref()).await.context(IoErr {
            path: path.as_ref(),
        })?;
        let datafile = no_intro::DataFile::from_file(s.as_str()).context(DatabaseReadErr {})?;
        Ok(Self(datafile))
    }

    pub fn contains(&self, file: &str) -> bool {
        self.0.games.iter().any(|game| game.name.as_str() == file)
    }

    pub fn similar_to<'s, 'a: 's>(&'s self, tokens: &'a Tokens<'a>) -> Vec<&'s str> {
        let mut similarities = self
            .0
            .games
            .iter()
            .filter_map(|game| {
                let game_tokens = Tokens::from_str(game.name.as_str());
                let same_words = tokens.words_in_common_with(&game_tokens);

                if game_tokens.word_count() == 1 && same_words >= 1 || same_words >= 2 {
                    Some((same_words, game))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        similarities.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        similarities
            .iter()
            .take(5)
            .map(|(_, game)| game.name.as_str())
            .collect()
    }
}

pub async fn load_all(args: &Args, config: &Config) -> Result<HashMap<String, Database>> {
    let path = args.resolve_path(config.db_dir());
    let mut readdir = read_dir(path.as_path()).await.context(IoErr {
        path: path.as_path(),
    })?;
    let mut databases = HashMap::new();

    loop {
        let entry = readdir.next_entry().await.context(IoErr {
            path: path.as_path(),
        })?;

        match entry {
            Some(entry) => {
                let path = entry.path();
                let system = path
                    .as_path()
                    .file_stem()
                    .and_then(|f| f.to_str())
                    .map(|s| s.to_owned())
                    .context(DatabaseNameErr {
                        path: path.as_path(),
                    })?;

                let db = Database::from_file(path).await?;
                databases.insert(system, db);
            }
            None => break,
        }
    }

    Ok(databases)
}

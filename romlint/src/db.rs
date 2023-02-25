use crate::error::Error;
use crate::word_match::Tokens;
use no_intro::DataFile;
use std::path::Path;
use tokio::fs::read_to_string;

pub struct Database(DataFile);

impl Database {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let s = read_to_string(path).await.map_err(Error::Io)?;
        let datafile = no_intro::DataFile::from_file(s.as_str())
            .map_err(|err| Error::Deserialize(err.to_string()))?;
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

                if game_tokens.word_count() == 1 && same_words >= 1 {
                    Some((same_words, game))
                } else if same_words >= 2 {
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

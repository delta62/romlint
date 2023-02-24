use crate::word_match::Tokens;
use serde::Deserialize;
use serde_xml_rs::from_str;
use std::{fs::read_to_string, path::Path};

#[derive(Debug, Deserialize)]
pub struct DataFile {
    header: Header,
    #[serde(rename = "game")]
    games: Vec<Game>,
}

#[derive(Debug, Deserialize)]
pub enum DataFileItem {
    Game(Game),
    Header(Header),
}

#[derive(Debug, Deserialize)]
pub struct Header {
    id: usize,
    name: String,
    description: String,
    version: String,
    author: String,
    homepage: String,
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct Game {
    name: String,
    description: String,
    rom: Rom,
}

#[derive(Debug, Deserialize)]
pub struct Rom {
    name: String,
    size: usize,
    crc: String,
}

impl DataFile {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let file = read_to_string(path).unwrap();
        from_str(file.as_str()).unwrap()
    }

    pub fn contains(&self, file: &str) -> bool {
        self.games.iter().any(|game| game.name.as_str() == file)
    }

    pub fn similar_to<'s, 'a: 's>(&'s self, tokens: &'a Tokens<'a>) -> Vec<&'s str> {
        let mut similarities = self
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

use serde::Deserialize;
use std::error;
use std::fmt::{self, Display, Formatter};

// Derived from https://datomatic.no-intro.org/stuff/schema_nointro_datfile_v2.xsd

#[derive(Debug)]
pub struct Error {
    message: String,
    path: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.message, self.path)
    }
}

impl error::Error for Error {}

#[derive(Debug, Deserialize)]
pub struct DataFile {
    pub header: Header,
    #[serde(rename = "game")]
    pub games: Vec<Game>,
}

#[derive(Debug, Deserialize)]
pub enum DataFileItem {
    Game(Game),
    Header(Header),
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub homepage: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Game {
    pub name: String,
    pub description: String,
}

impl DataFile {
    pub fn from_file(s: &str) -> Result<Self, Error> {
        let bytes = s.as_bytes();
        let mut de = serde_xml_rs::Deserializer::new_from_reader(bytes);
        serde_path_to_error::deserialize(&mut de).map_err(|err| Error {
            path: err.path().to_string(),
            message: err.to_string(),
        })
    }
}

use serde::Deserialize;
use std::fmt::Display;

// Derived from https://datomatic.no-intro.org/stuff/schema_nointro_datfile_v2.xsd

#[derive(Debug)]
pub struct Error {
    message: String,
    path: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.message, self.path)
    }
}

impl std::error::Error for Error {}

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
    pub id: usize,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub homepage: String,
    pub url: String,
    pub clrmamepro: ClrMamePro,
}

#[derive(Debug, Deserialize)]
pub struct ClrMamePro {
    pub forcenodump: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Game {
    pub name: String,
    pub description: String,
    pub rom: Rom,
}

#[derive(Debug, Deserialize)]
pub struct Rom {
    pub name: String,
    pub size: usize,
    pub crc: String,
    pub md5: String,
    pub sha1: String,
    pub sha256: Option<String>,
    pub status: Option<String>,
    pub serial: Option<String>,
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

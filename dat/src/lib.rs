use serde::Deserialize;
use std::error;
use std::fmt::{self, Display, Formatter};

// Derived from https://datomatic.no-intro.org/stuff/schema_nointro_datfile_v3.xsd

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct DataFile {
    pub header: Header,
    #[serde(rename = "game")]
    pub games: Vec<Game>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Header {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub homepage: String,
    pub url: String,
    pub subset: Option<String>,
    pub clrmamepro: ClrMamePro,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum ForceNoDump {
    #[serde(rename = "obsolete")]
    Obsolete,
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "ignore")]
    Ignore,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ClrMamePro {
    #[serde(rename = "forcenodump")]
    pub force_no_dump: Option<ForceNoDump>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Rom {
    pub name: String,
    pub size: usize,
    pub crc: String,
    pub md5: String,
    pub sha1: String,
    pub sha256: Option<String>,
    pub status: Option<String>,
    pub serial: Option<String>,
    pub header: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Game {
    pub name: String,
    pub description: String,
    pub rom: Rom,
}

impl DataFile {
    pub fn from_file(s: &str) -> Result<Self, Error> {
        let bytes = s.as_bytes();
        let mut de = serde_xml_rs::Deserializer::new_from_reader(bytes);
        let mut data_file: DataFile =
            serde_path_to_error::deserialize(&mut de).map_err(|err| Error {
                path: err.path().to_string(),
                message: err.to_string(),
            })?;

        data_file.games.shrink_to_fit();

        Ok(data_file)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn parses_header_id() {
        let dat = v3_sample();
        assert_eq!(dat.header.id, 42);
    }

    #[test]
    fn parses_header_name() {
        let dat = v3_sample();
        assert_eq!(dat.header.name, "Rust DAT parser");
    }

    #[test]
    fn parses_header_description() {
        let dat = v3_sample();
        assert_eq!(dat.header.description, "Test input file");
    }

    #[test]
    fn parses_header_version() {
        let dat = v3_sample();
        assert_eq!(dat.header.version, "20240830-122750");
    }

    #[test]
    fn parses_header_author() {
        let dat = v3_sample();
        assert_eq!(dat.header.author, "delta62");
    }

    #[test]
    fn parses_header_homepage() {
        let dat = v3_sample();
        assert_eq!(dat.header.homepage, "My homepage");
    }

    #[test]
    fn parses_header_url() {
        let dat = v3_sample();
        assert_eq!(dat.header.url, "https://example.org");
    }

    #[test]
    fn parses_header_clrmamepro() {
        let dat = v3_sample();
        assert_eq!(
            dat.header.clrmamepro,
            ClrMamePro {
                force_no_dump: Some(ForceNoDump::Required)
            }
        );
    }

    #[test]
    fn parses_game_name() {
        let dat = v3_sample();
        let game = dat.games.first().unwrap();
        assert_eq!(game.name, "Example Game");
    }

    #[test]
    fn parses_game_description() {
        let dat = v3_sample();
        let game = dat.games.first().unwrap();
        assert_eq!(game.description, "Example Game, released in 2024");
    }

    #[test]
    fn parses_rom_size() {
        let dat = v3_sample();
        let game = dat.games.first().unwrap();
        assert_eq!(game.rom.size, 5432);
    }

    #[test]
    fn parses_rom_crc() {
        let dat = v3_sample();
        let game = dat.games.first().unwrap();
        assert_eq!(game.rom.crc, "148323542");
    }

    #[test]
    fn parses_rom_md5() {
        let dat = v3_sample();
        let game = dat.games.first().unwrap();
        assert_eq!(game.rom.md5, "1a79a4d60de6718e8e5b326e338ae533");
    }

    #[test]
    fn parses_rom_sha1() {
        let dat = v3_sample();
        let game = dat.games.first().unwrap();
        assert_eq!(game.rom.sha1, "c3499c2729730a7f807efb8676a92dcb6f8a3f8f");
    }

    #[test]
    fn parses_rom_sha256() {
        let dat = v3_sample();
        let game = dat.games.first().unwrap();
        assert_eq!(
            game.rom.sha256,
            Some("50d858e0985ecc7f60418aaf0cc5ab587f42c2570a884095a9e8ccacd0f6545c".to_owned())
        );
    }

    fn v3_sample() -> DataFile {
        let txt = read_to_string("samples/v3.dat").unwrap();
        DataFile::from_file(&txt).unwrap()
    }
}

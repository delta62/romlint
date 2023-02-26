use std::{error, fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    Deserialize(String),
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deserialize(err) => write!(f, "{}", err),
            Self::Io(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

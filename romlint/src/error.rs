use std::{error, fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    Deserialize(String),
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

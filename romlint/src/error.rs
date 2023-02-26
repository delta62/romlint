use snafu::prelude::*;
use std::{io, path::PathBuf};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub), context(suffix(Err)))]
pub enum Error {
    #[snafu(display("error reading ROM database"))]
    DatabaseRead { source: no_intro::Error },

    #[snafu(display("unable to determine the system name of {}", path.display()))]
    DatabaseName { path: PathBuf },

    #[snafu(display("error reading config"))]
    ConfigRead { source: toml::de::Error },

    #[snafu(display("error accessing {}", path.display()))]
    Io { path: PathBuf, source: io::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

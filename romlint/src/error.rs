use crate::ui::Message;
use snafu::prelude::*;
use std::{io, path::PathBuf, sync::mpsc::SendError};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub), context(suffix(Err)))]
pub enum Error {
    #[snafu(display("error reading {} database: {source}", path.display()))]
    DatabaseRead { path: PathBuf, source: dat::Error },

    #[snafu(display("unable to determine the system name of {}", path.display()))]
    DatabaseName { path: PathBuf },

    #[snafu(display("error reading config: {source}"))]
    ConfigRead { source: toml::de::Error },

    #[snafu(display("error accessing {}", path.display()))]
    Io { path: PathBuf, source: io::Error },

    #[snafu(display("unable to process path {}", path.display()))]
    InvalidPath { path: PathBuf },

    #[snafu(display("attempted to send over a broken pipe"))]
    BrokenPipe { source: SendError<Message> },

    #[snafu(display("attempted to read the parent of {}", path.display()))]
    NoParent { path: PathBuf },
}

pub type Result<T> = std::result::Result<T, Error>;

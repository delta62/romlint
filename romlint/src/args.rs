use crate::error::{NoParentErr, Result};
use clap::Parser;
use snafu::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    /// The directory to use as the working directory when searching for roms and the
    /// config file
    #[clap(short, long)]
    cwd: Option<String>,

    /// The path (relative to the current working directory) where a romlint.toml
    /// config file is located
    #[clap(long, default_value = "romlint.toml")]
    pub config_path: String,

    /// Override the system to run lint rules for. If not specified, roms are assumed
    /// to exist inside of directories which share a name with their system.
    #[clap(short, long)]
    pub system: Option<String>,

    /// Skip the archive_file_name rule
    #[clap(long, default_value_t = false)]
    pub no_archived_file_name: bool,
}

impl Args {
    pub fn cwd(&self) -> PathBuf {
        self.cwd
            .as_ref()
            .map(|c| Path::new(c.as_str()).to_path_buf())
            .or_else(|| std::env::current_dir().ok())
            .expect("Unable to access current working directory")
    }

    pub fn config_path(&self) -> PathBuf {
        self.cwd().join(self.config_path.as_str())
    }

    pub fn config_dir(&self) -> Result<PathBuf> {
        let path = self.config_path();

        path.as_path()
            .parent()
            .map(|path| path.to_path_buf())
            .context(NoParentErr { path })
    }
}

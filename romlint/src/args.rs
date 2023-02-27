use crate::error::{NoParentErr, Result};
use clap::Parser;
use snafu::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    cwd: Option<String>,

    #[clap(long, default_value = "romlint.toml")]
    pub config_path: String,

    #[clap(short, long)]
    pub system: Option<String>,
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

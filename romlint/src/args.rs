use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    cwd: Option<String>,

    #[clap(long, default_value = "romlint.toml")]
    pub config_path: String,

    #[clap(short, long)]
    pub system: String,
}

impl Args {
    pub fn resolve_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        if let Some(cwd) = self.cwd.as_ref() {
            Path::new(cwd.as_str()).join(path)
        } else {
            Path::new(path.as_ref()).to_path_buf()
        }
    }

    pub fn cwd(&self) -> PathBuf {
        self.cwd
            .as_ref()
            .map(|c| Path::new(c.as_str()).to_path_buf())
            .or_else(|| std::env::current_dir().ok())
            .expect("Unable to access current working directory")
    }
}

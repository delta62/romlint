use clap::{Args as ClapArgs, Parser, Subcommand};
use std::path::{Path, PathBuf};

/// A tool for enumerating and keeping ROMs organized
#[derive(Clone, Debug, Parser)]
#[clap(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

    /// Set the working directory to use. If unset, defaults to the current working
    /// directory
    #[clap(short, long)]
    cwd: Option<String>,

    /// Override the system to run against. If unspecified, all systems are used
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
        self.cwd().join("romlint.toml")
    }

    pub fn system_path(&self, system: &str) -> PathBuf {
        self.cwd().join(system)
    }
}

/// The command to run
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Dump all known ROM names to stdout
    Dump,
    /// List local ROM names to stdout
    Inventory,
    /// Run lints against local ROMs
    Lint(LintArgs),
}

#[derive(Clone, Debug, ClapArgs)]
pub struct LintArgs {
    /// Only show output for files which are failing lints
    #[clap(long, default_value_t = false)]
    pub hide_passes: bool,

    /// Only lint the given file. If omitted, all files are linted.
    pub file: Option<String>,
}

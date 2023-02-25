use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub enum Command {
    Scan,
    Watch,
}

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    pub cwd: String,

    #[clap(short, long)]
    pub db: String,

    pub command: Command,
}

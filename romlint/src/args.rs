use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    pub cwd: String,

    #[clap(short, long)]
    pub db: String,
}

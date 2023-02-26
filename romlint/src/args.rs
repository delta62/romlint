use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    pub cwd: String,

    #[clap(short, long)]
    pub db: String,

    #[clap(short, long)]
    pub system: String,
}

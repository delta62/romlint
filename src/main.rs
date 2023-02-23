mod args;
mod dir_walker;
mod linter;

use args::Args;
use clap::Parser;
use dir_walker::walk;
use futures::TryStreamExt;
use linter::{NoArchives, NoJunkFiles, Rule};
use std::path::PathBuf;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let rules: Vec<Box<dyn Rule>> = vec![Box::new(NoJunkFiles), Box::new(NoArchives)];
    let path = PathBuf::from(args.cwd.as_str());
    let mut stream = Box::pin(walk(path).await.unwrap());

    loop {
        let next = stream.try_next().await;
        match next {
            Ok(Some(entry)) => {
                let diag = rules.iter().find_map(|rule| rule.check(&entry));
                if let Some(diag) = diag {
                    println!(
                        "{} - {}",
                        diag.path
                            .strip_prefix(args.cwd.as_str())
                            .ok()
                            .and_then(|p| p.to_str())
                            .unwrap_or_default(),
                        diag.message,
                    );
                }
            }
            Ok(None) => break,
            Err(err) => {
                panic!("{}", err);
            }
        }
    }
}

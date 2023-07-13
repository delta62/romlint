use crate::{args::Args, config::Config, db, error::Result, ui::Message};

/// Dump all known ROM names to stdout. Each name is printed on a separate line.
pub async fn dump(args: Args) -> Result<()> {
    let config = Config::from_path(args.config_path()).await?;
    let db_path = args.cwd().join(config.db_dir());
    let dbs;

    if let Some(sys) = args.system {
        dbs = db::load_only(&db_path, &[&sys], &nop).await?;

        if dbs.is_empty() {
            eprint!("Unable to find a database for the system '{sys}'.");
        }
    } else {
        dbs = db::load_all(&db_path, &nop).await?;
    }

    dbs.iter()
        .flat_map(|db| db.files())
        .for_each(|file| println!("{}", file.name));

    Ok(())
}

fn nop(_message: Message) -> Result<()> {
    Ok(())
}

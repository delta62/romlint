use crate::{args::Args, config, db, error::Result};

pub async fn dump(args: Args) -> Result<()> {
    let config = config::from_path(args.config_path()).await?;
    let db_path = args.cwd().join(config.db_dir());
    let dbs;

    if let Some(sys) = args.system {
        dbs = db::load_only(&db_path, &[&sys], None).await?;

        if dbs.is_empty() {
            eprint!("Unable to find a database for the system '{sys}'.");
        }
    } else {
        dbs = db::load_all(&db_path, None).await?;
    }

    for db in dbs.values() {
        for file in db.files() {
            println!("{}", file.name);
        }
    }

    Ok(())
}

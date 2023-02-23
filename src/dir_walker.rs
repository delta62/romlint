use futures::{stream::once, Stream, StreamExt, TryStreamExt};
use std::{
    fs::Metadata,
    io::Result,
    path::{Path, PathBuf},
};
use tokio::fs::{metadata, read_dir, DirEntry};
use tokio_stream::wrappers::ReadDirStream;

type MetaResult = Result<FileMeta>;

pub struct FileMeta {
    pub meta: Metadata,
    pub entry: DirEntry,
}

impl FileMeta {
    async fn from_dir_entry(entry: DirEntry) -> Result<Self> {
        let meta = metadata(entry.path()).await?;
        Ok(Self { meta, entry })
    }
}

pub async fn walk(path: PathBuf) -> Result<impl Stream<Item = MetaResult>> {
    Ok(dir(path)
        .await?
        .and_then(dir_or_singleton)
        .boxed()
        .try_flatten())
}

pub async fn dir<P: AsRef<Path>>(path: P) -> Result<impl Stream<Item = MetaResult>> {
    let entries = read_dir(path).await?;
    Ok(ReadDirStream::new(entries).and_then(FileMeta::from_dir_entry))
}

async fn dir_or_singleton(file: FileMeta) -> Result<impl Stream<Item = MetaResult>> {
    let is_dir = file.meta.is_dir();

    if is_dir {
        let path = file.entry.path();
        let rest = dir(path).await?;
        let stream = once(async { Ok(file) });
        Ok(stream.chain(rest).boxed())
    } else {
        Ok(once(async { Ok(file) }).boxed())
    }
}

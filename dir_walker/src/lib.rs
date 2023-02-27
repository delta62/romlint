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
    pub depth: usize,
    pub meta: Metadata,
    pub path: PathBuf,
}

impl FileMeta {
    async fn from_dir_entry(entry: DirEntry, depth: usize) -> Result<Self> {
        let path = entry.path();
        let meta = metadata(&path).await?;

        Ok(Self { meta, depth, path })
    }
}

pub async fn walk<P: Into<PathBuf>>(path: P) -> Result<impl Stream<Item = MetaResult>> {
    Ok(dir(path.into(), 0)
        .await?
        .and_then(dir_or_singleton)
        .boxed()
        .try_flatten())
}

async fn dir<P: AsRef<Path>>(path: P, depth: usize) -> Result<impl Stream<Item = MetaResult>> {
    let entries = read_dir(path).await?;
    Ok(ReadDirStream::new(entries).and_then(move |entry| FileMeta::from_dir_entry(entry, depth)))
}

async fn dir_or_singleton(file: FileMeta) -> Result<impl Stream<Item = MetaResult>> {
    let is_dir = file.meta.is_dir();

    if is_dir {
        let path = file.path.clone();
        let rest = dir(path, file.depth + 1).await?;
        let stream = once(async { Ok(file) });
        Ok(stream.chain(rest).boxed())
    } else {
        Ok(once(async { Ok(file) }).boxed())
    }
}

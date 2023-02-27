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
    pub async fn from_path<P: AsRef<Path>>(path: P, depth: usize) -> Result<Self> {
        let meta = metadata(path.as_ref()).await?;
        let path = path.as_ref().to_path_buf();

        Ok(Self { meta, path, depth })
    }

    async fn from_dir_entry(entry: DirEntry, depth: usize) -> Result<Self> {
        let path = entry.path();
        let meta = metadata(&path).await?;

        Ok(Self { meta, depth, path })
    }
}

pub async fn walk(path: PathBuf) -> Result<impl Stream<Item = MetaResult>> {
    Ok(dir(path, 0)
        .await?
        .and_then(move |entry| dir_or_singleton(entry, 1))
        .boxed()
        .try_flatten())
}

async fn dir<P: AsRef<Path>>(path: P, depth: usize) -> Result<impl Stream<Item = MetaResult>> {
    let entries = read_dir(path).await?;
    Ok(ReadDirStream::new(entries)
        .and_then(move |entry| FileMeta::from_dir_entry(entry, depth + 1)))
}

async fn dir_or_singleton(file: FileMeta, depth: usize) -> Result<impl Stream<Item = MetaResult>> {
    let is_dir = file.meta.is_dir();

    if is_dir {
        let path = file.path.clone();
        let rest = dir(path, depth + 1).await?;
        let stream = once(async { Ok(file) });
        Ok(stream.chain(rest).boxed())
    } else {
        Ok(once(async { Ok(file) }).boxed())
    }
}

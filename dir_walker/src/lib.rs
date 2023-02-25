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
    meta: Metadata,
    path: PathBuf,
}

impl FileMeta {
    async fn from_dir_entry(entry: DirEntry) -> Result<Self> {
        let path = entry.path();
        let meta = metadata(&path).await?;

        Ok(Self { meta, path })
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn extension(&self) -> Option<&str> {
        self.path.as_path().extension().and_then(|e| e.to_str())
    }

    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }
}

pub async fn walk(path: PathBuf) -> Result<impl Stream<Item = MetaResult>> {
    Ok(dir(path)
        .await?
        .and_then(dir_or_singleton)
        .boxed()
        .try_flatten())
}

async fn dir<P: AsRef<Path>>(path: P) -> Result<impl Stream<Item = MetaResult>> {
    let entries = read_dir(path).await?;
    Ok(ReadDirStream::new(entries).and_then(FileMeta::from_dir_entry))
}

async fn dir_or_singleton(file: FileMeta) -> Result<impl Stream<Item = MetaResult>> {
    let is_dir = file.meta.is_dir();

    if is_dir {
        let path = file.path().to_path_buf();
        let rest = dir(path).await?;
        let stream = once(async { Ok(file) });
        Ok(stream.chain(rest).boxed())
    } else {
        Ok(once(async { Ok(file) }).boxed())
    }
}

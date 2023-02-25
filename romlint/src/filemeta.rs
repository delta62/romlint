use dir_walker::FileMeta as DirMeta;
use std::fs::Metadata;
use std::{
    io::Error,
    path::{Path, PathBuf},
};
use tokio::task::spawn_blocking;
use zip::ZipArchive;

pub struct ArchiveInfo {
    file_names: Vec<String>,
}

impl ArchiveInfo {
    pub fn len(&self) -> usize {
        self.file_names.len()
    }
}

pub struct FileMeta {
    meta: Metadata,
    path: PathBuf,
    archive: Option<ArchiveInfo>,
}

impl FileMeta {
    pub async fn from_dir_walker(file: DirMeta) -> Result<Self, Error> {
        let archive = if let Some("zip") = file.path.extension().and_then(|os| os.to_str()) {
            let path = file.path.clone();
            spawn_blocking(|| {
                let handle = std::fs::File::open(path)?;
                let reader = std::io::BufReader::new(handle);
                let arc = ZipArchive::new(reader).unwrap();

                let file_names = arc.file_names().map(|s| s.to_owned()).collect();

                Ok::<Option<ArchiveInfo>, std::io::Error>(Some(ArchiveInfo { file_names }))
            })
            .await??
        } else {
            None
        };

        Ok(Self {
            archive,
            meta: file.meta,
            path: file.path,
        })
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

    pub fn archive(&self) -> Option<&ArchiveInfo> {
        self.archive.as_ref()
    }
}

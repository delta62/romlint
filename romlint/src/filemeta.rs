use crate::config::{Config, ResolvedConfig};
use dir_walker::FileMeta as DirMeta;
use std::fs::{File, Metadata};
use std::{
    io::{BufReader, Result},
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

    pub fn file_names(&self) -> impl Iterator<Item = &str> {
        self.file_names.iter().map(|s| s.as_str())
    }
}

pub struct FileMeta<'a> {
    config: ResolvedConfig<'a>,
    meta: Metadata,
    path: PathBuf,
    archive: Option<ArchiveInfo>,
    system: String,
}

impl<'a> FileMeta<'a> {
    pub async fn from_dir_walker<'b: 'a>(
        file: DirMeta,
        system: &str,
        config: &'b Config,
    ) -> Result<FileMeta<'a>> {
        let extension = file.path.extension().and_then(|os| os.to_str());
        let mut archive = None;
        let config = config.resolve(system).unwrap();

        if let Some("zip") = extension {
            let path = file.path.clone();
            let info = spawn_blocking(|| -> Result<Option<ArchiveInfo>> {
                let file = File::open(path)?;
                let reader = BufReader::new(file);
                let zip = ZipArchive::new(reader).unwrap();
                let file_names = zip.file_names().map(|s| s.to_owned()).collect();

                Ok(Some(ArchiveInfo { file_names }))
            })
            .await??;
            archive = info;
        };

        Ok(Self {
            archive,
            config,
            meta: file.meta,
            path: file.path,
            system: system.to_owned(),
        })
    }

    pub fn config(&self) -> &ResolvedConfig {
        &self.config
    }

    pub fn basename(&self) -> Option<&str> {
        self.path.file_stem().and_then(|s| s.to_str())
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

    pub fn system(&self) -> &str {
        self.system.as_str()
    }
}

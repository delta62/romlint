use crate::config::{Config, ResolvedConfig};
use dir_walker::FileMeta as DirMeta;
use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::{
    io::{BufReader, Result},
    path::{Path, PathBuf},
};
use tokio::fs::metadata;
use zip::ZipArchive;

pub struct ArchiveInfo {
    file_names: Vec<String>,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
}

impl ArchiveInfo {
    pub fn file_names(&self) -> impl Iterator<Item = &Path> {
        self.file_names.iter().map(Path::new)
    }
}

pub struct FileMeta<'a> {
    archive: Option<ArchiveInfo>,
    config: Option<ResolvedConfig<'a>>,
    depth: usize,
    forced_system: Option<&'a str>,
    meta: Metadata,
    path: PathBuf,
}

fn system_from_path(path: &Path) -> Option<&str> {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
}

pub trait Extractor {
    fn extract(&self, path: &Path) -> Result<ArchiveInfo>;
}

pub struct ZipExtractor;

impl Extractor for ZipExtractor {
    fn extract(&self, path: &Path) -> Result<ArchiveInfo> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut zip = ZipArchive::new(reader).unwrap();
        let mut uncompressed_size = 0;
        let mut compressed_size = 0;
        let mut file_names = Vec::with_capacity(zip.len());

        for i in 0..zip.len() {
            let file = zip.by_index_raw(i).unwrap();
            uncompressed_size += file.size();
            compressed_size += file.compressed_size();
            file_names.push(file.name().to_string());
        }

        Ok(ArchiveInfo {
            file_names,
            uncompressed_size,
            compressed_size,
        })
    }
}

impl<'a> FileMeta<'a> {
    async fn from_raw_parts<'b: 'a, P: AsRef<Path>>(
        system: Option<&'b str>,
        config: &'b Config,
        path: P,
        meta: Option<Metadata>,
        depth: usize,
        extractors: &HashMap<String, Box<dyn Extractor>>,
    ) -> Result<FileMeta<'a>> {
        let path = path.as_ref();
        let config = system
            .or_else(|| system_from_path(path))
            .and_then(|sys| config.resolve(sys));

        let meta = if let Some(metadata) = meta {
            metadata
        } else {
            metadata(path).await?
        };

        let archive = path
            .extension()
            .and_then(|os| os.to_str())
            .and_then(|ext| extractors.get(ext))
            .map(|extractor| extractor.extract(path));

        let archive = match archive {
            Some(Ok(archive)) => Some(archive),
            Some(Err(err)) => Err(err)?,
            None => None,
        };

        Ok(Self {
            archive,
            config,
            depth,
            forced_system: system,
            path: path.to_path_buf(),
            meta,
        })
    }

    pub async fn from_path<'b: 'a, P: AsRef<Path>>(
        system: Option<&'b str>,
        config: &'b Config,
        path: P,
        extractors: &HashMap<String, Box<dyn Extractor>>,
    ) -> Result<FileMeta<'a>> {
        let metadata = None;
        let depth = 1;
        Self::from_raw_parts(system, config, path, metadata, depth, extractors).await
    }

    pub async fn from_dir_walker<'b: 'a>(
        file: DirMeta,
        system: Option<&'b str>,
        config: &'b Config,
        extractors: &HashMap<String, Box<dyn Extractor>>,
    ) -> Result<FileMeta<'a>> {
        let path = file.path.as_path();
        let meta = Some(file.meta);
        let depth = file.depth;

        Self::from_raw_parts(system, config, path, meta, depth, extractors).await
    }

    pub fn config(&self) -> Option<&ResolvedConfig> {
        self.config.as_ref()
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    pub fn archive(&self) -> Option<&ArchiveInfo> {
        self.archive.as_ref()
    }

    pub fn system(&self) -> Option<&str> {
        if self.forced_system.is_some() {
            self.forced_system
        } else if self.depth != 1 {
            None
        } else {
            system_from_path(&self.path)
        }
    }
}

use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::{Path, PathBuf};

use zip::read::ZipArchive;

use crate::ViewerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub name: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
}

#[derive(Debug, Clone)]
pub enum ArchiveSource {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

impl ArchiveSource {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        Self::Path(path.as_ref().to_path_buf())
    }

    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self::Bytes(bytes.into())
    }
}

#[derive(Debug, Clone)]
pub struct OoxmlArchive {
    source: ArchiveSource,
    entries: Vec<ArchiveEntry>,
}

impl OoxmlArchive {
    pub fn open_path(path: impl AsRef<Path>) -> Result<Self, ViewerError> {
        Self::open(ArchiveSource::from_path(path))
    }

    pub fn open_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self, ViewerError> {
        Self::open(ArchiveSource::from_bytes(bytes))
    }

    pub fn open(source: ArchiveSource) -> Result<Self, ViewerError> {
        let mut zip = zip_from_source(&source)?;
        let mut entries = Vec::with_capacity(zip.len());

        for index in 0..zip.len() {
            let entry = zip.by_index(index).map_err(|_| ViewerError::InvalidDocument)?;
            entries.push(ArchiveEntry {
                name: entry.name().to_string(),
                compressed_size: entry.compressed_size(),
                uncompressed_size: entry.size(),
            });
        }

        Ok(Self { source, entries })
    }

    pub fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    pub fn contains_part(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.name == name)
    }

    pub fn entry(&self, name: &str) -> Option<&ArchiveEntry> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    pub fn read_part(&self, name: &str) -> Result<Vec<u8>, ViewerError> {
        let mut zip = zip_from_source(&self.source)?;
        let mut part = zip.by_name(name).map_err(|_| ViewerError::InvalidDocument)?;
        let mut bytes = Vec::with_capacity(part.size() as usize);
        part.read_to_end(&mut bytes)
            .map_err(|error| ViewerError::Io(error.to_string()))?;
        Ok(bytes)
    }
}

fn zip_from_source(source: &ArchiveSource) -> Result<ZipArchive<Box<dyn ReadSeek>>, ViewerError> {
    match source {
        ArchiveSource::Path(path) => {
            let file = File::open(path).map_err(|error| ViewerError::Io(error.to_string()))?;
            ZipArchive::new(Box::new(file) as Box<dyn ReadSeek>)
                .map_err(|_| ViewerError::InvalidDocument)
        }
        ArchiveSource::Bytes(bytes) => {
            let cursor = Cursor::new(bytes.clone());
            ZipArchive::new(Box::new(cursor) as Box<dyn ReadSeek>)
                .map_err(|_| ViewerError::InvalidDocument)
        }
    }
}

trait ReadSeek: Read + Seek {}
impl<T> ReadSeek for T where T: Read + Seek {}

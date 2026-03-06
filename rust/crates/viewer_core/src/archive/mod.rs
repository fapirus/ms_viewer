use std::fs::File;
use std::io::Read;
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
pub struct OoxmlArchive {
    path: PathBuf,
    entries: Vec<ArchiveEntry>,
}

impl OoxmlArchive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ViewerError> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path).map_err(|error| ViewerError::Io(error.to_string()))?;
        let mut zip = ZipArchive::new(file).map_err(|_| ViewerError::InvalidDocument)?;
        let mut entries = Vec::with_capacity(zip.len());

        for index in 0..zip.len() {
            let entry = zip.by_index(index).map_err(|_| ViewerError::InvalidDocument)?;
            entries.push(ArchiveEntry {
                name: entry.name().to_string(),
                compressed_size: entry.compressed_size(),
                uncompressed_size: entry.size(),
            });
        }

        Ok(Self { path, entries })
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
        let file = File::open(&self.path).map_err(|error| ViewerError::Io(error.to_string()))?;
        let mut zip = ZipArchive::new(file).map_err(|_| ViewerError::InvalidDocument)?;
        let mut part = zip.by_name(name).map_err(|_| ViewerError::InvalidDocument)?;
        let mut bytes = Vec::with_capacity(part.size() as usize);
        part.read_to_end(&mut bytes)
            .map_err(|error| ViewerError::Io(error.to_string()))?;
        Ok(bytes)
    }
}

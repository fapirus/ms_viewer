use std::fs;
use std::path::Path;

use crate::ViewerError;

const ZIP_MAGIC: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
const OLE_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageKind {
    Plain,
    Encrypted,
    UnsupportedEncryption,
}

pub fn detect_package_kind_from_path(path: impl AsRef<Path>) -> Result<PackageKind, ViewerError> {
    let bytes = fs::read(path).map_err(|error| ViewerError::Io(error.to_string()))?;
    detect_package_kind_from_bytes(&bytes)
}

pub fn detect_package_kind_from_bytes(bytes: &[u8]) -> Result<PackageKind, ViewerError> {
    if bytes.starts_with(&ZIP_MAGIC) {
        return Ok(PackageKind::Plain);
    }

    if bytes.starts_with(&OLE_MAGIC) {
        let has_encrypted_package = contains_ascii(bytes, b"EncryptedPackage");
        let has_encryption_info = contains_ascii(bytes, b"EncryptionInfo");

        if has_encrypted_package && has_encryption_info {
            return Ok(PackageKind::Encrypted);
        }

        return Ok(PackageKind::UnsupportedEncryption);
    }

    Err(ViewerError::InvalidDocument)
}

fn contains_ascii(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

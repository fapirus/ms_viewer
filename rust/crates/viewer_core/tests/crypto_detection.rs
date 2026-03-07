use std::fs;
use std::io::Write;

use tempfile::tempdir;
use viewer_core::crypto::{
    detect_package_kind_from_bytes, detect_package_kind_from_path, PackageKind,
};
use viewer_core::ViewerError;
use zip::write::SimpleFileOptions;

fn create_zip(path: &std::path::Path) {
    let file = fs::File::create(path).expect("zip file should be created");
    let mut writer = zip::ZipWriter::new(file);
    writer
        .start_file("word/document.xml", SimpleFileOptions::default())
        .expect("zip entry should start");
    writer
        .write_all(b"<w:document />")
        .expect("zip bytes should write");
    writer.finish().expect("zip should finish");
}

#[test]
fn detects_plain_zip_package() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("plain.docx");
    create_zip(&path);

    let kind = detect_package_kind_from_path(&path).expect("plain zip should detect");

    assert_eq!(kind, PackageKind::Plain);
}

#[test]
fn detects_encrypted_ole_package() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("encrypted.docx");
    let mut bytes = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    bytes.extend_from_slice(b"...EncryptionInfo...EncryptedPackage...");
    fs::write(&path, &bytes).expect("encrypted fixture should write");

    let kind = detect_package_kind_from_path(&path).expect("encrypted fixture should detect");

    assert_eq!(kind, PackageKind::Encrypted);
}

#[test]
fn detects_unsupported_encryption_container() {
    let bytes = [
        0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, b'N', b'o', b'S', b'u', b'p',
    ];

    let kind = detect_package_kind_from_bytes(&bytes).expect("ole bytes should classify");

    assert_eq!(kind, PackageKind::UnsupportedEncryption);
}

#[test]
fn invalid_bytes_return_invalid_document() {
    let error =
        detect_package_kind_from_bytes(b"not-a-doc").expect_err("invalid bytes should fail");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

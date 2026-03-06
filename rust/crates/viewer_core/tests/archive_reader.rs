use std::fs;
use std::io::Write;

use tempfile::tempdir;
use viewer_core::archive::OoxmlArchive;
use viewer_core::ViewerError;
use zip::write::SimpleFileOptions;

fn create_zip(path: &std::path::Path, entries: &[(&str, &[u8])]) {
    let file = fs::File::create(path).expect("zip file should be created");
    let mut writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    for (name, bytes) in entries {
        writer
            .start_file(name, options)
            .expect("file entry should start");
        writer.write_all(bytes).expect("entry bytes should write");
    }

    writer.finish().expect("zip should finish");
}

#[test]
fn opens_valid_zip_and_lists_parts() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sample.docx");
    create_zip(
        &path,
        &[
            ("[Content_Types].xml", b"<Types />"),
            ("word/document.xml", b"<w:document />"),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");

    assert!(archive.contains_part("word/document.xml"));
    assert_eq!(archive.entries().len(), 2);
}

#[test]
fn same_document_opens_from_path_and_bytes() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sample.docx");
    create_zip(
        &path,
        &[
            ("[Content_Types].xml", b"<Types />"),
            ("word/document.xml", b"hello world"),
        ],
    );

    let bytes = fs::read(&path).expect("archive bytes should read");
    let from_path = OoxmlArchive::open_path(&path).expect("path archive should open");
    let from_bytes = OoxmlArchive::open_bytes(bytes).expect("byte archive should open");

    assert_eq!(from_path.entries().len(), from_bytes.entries().len());
    assert_eq!(
        from_path.read_part("word/document.xml").expect("path part should read"),
        from_bytes
            .read_part("word/document.xml")
            .expect("byte part should read"),
    );
}

#[test]
fn missing_part_lookup_returns_none_and_read_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sample.docx");
    create_zip(&path, &[("word/document.xml", b"hello")]);

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");

    assert!(archive.entry("missing.xml").is_none());
    let error = archive.read_part("missing.xml").expect_err("missing part should fail");
    assert!(matches!(error, ViewerError::InvalidDocument));
}

#[test]
fn invalid_source_returns_invalid_source_error_mapping() {
    let error = OoxmlArchive::open_bytes(b"not a zip".to_vec()).expect_err("invalid bytes should fail");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

#[test]
fn invalid_zip_returns_invalid_document_error() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("broken.docx");
    fs::write(&path, b"not a zip").expect("file should write");

    let error = OoxmlArchive::open_path(&path).expect_err("invalid zip should fail");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

#[test]
fn large_entry_is_only_read_when_requested() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("large.docx");
    let large = vec![b'a'; 1024 * 1024];
    create_zip(&path, &[("word/document.xml", &large)]);

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let entry = archive
        .entry("word/document.xml")
        .expect("entry metadata should exist");

    assert_eq!(entry.uncompressed_size, large.len() as u64);

    let bytes = archive
        .read_part("word/document.xml")
        .expect("entry bytes should read only on demand");
    assert_eq!(bytes.len(), large.len());
}

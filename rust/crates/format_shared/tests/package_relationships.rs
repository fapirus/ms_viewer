use std::fs;
use std::io::Write;

use format_shared::{parse_shared_package, resolve_relationship_target};
use tempfile::tempdir;
use viewer_core::archive::OoxmlArchive;
use viewer_core::ViewerError;
use zip::write::SimpleFileOptions;

fn create_zip(path: &std::path::Path, entries: &[(&str, &[u8])]) {
    let file = fs::File::create(path).expect("zip file should be created");
    let mut writer = zip::ZipWriter::new(file);

    for (name, bytes) in entries {
        writer
            .start_file(name, SimpleFileOptions::default())
            .expect("entry should start");
        writer.write_all(bytes).expect("entry bytes should write");
    }

    writer.finish().expect("zip should finish");
}

#[test]
fn parses_package_relationships_and_content_types() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sample.docx");
    create_zip(
        &path,
        &[
            (
                "[Content_Types].xml",
                br#"<Types xmlns="urn:test"><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml" /></Types>"#,
            ),
            (
                "_rels/.rels",
                br#"<Relationships xmlns="urn:test"><Relationship Id="rId1" Type="officeDocument" Target="word/document.xml" /></Relationships>"#,
            ),
            ("word/document.xml", b"<w:document />"),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let model = parse_shared_package(&archive).expect("shared package should parse");

    assert_eq!(model.content_types.len(), 1);
    assert_eq!(model.relationships.len(), 1);
    assert_eq!(model.relationships[0].resolved_target, "/word/document.xml");
}

#[test]
fn missing_relationship_target_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sample.docx");
    create_zip(
        &path,
        &[
            (
                "[Content_Types].xml",
                br#"<Types xmlns="urn:test"><Override PartName="/word/document.xml" ContentType="application/test" /></Types>"#,
            ),
            (
                "_rels/.rels",
                br#"<Relationships xmlns="urn:test"><Relationship Id="rId1" Type="officeDocument" Target="word/missing.xml" /></Relationships>"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let error = parse_shared_package(&archive).expect_err("missing target should fail");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

#[test]
fn resolves_relative_relationship_targets() {
    assert_eq!(
        resolve_relationship_target("/word/_rels/document.xml.rels", "../media/image1.png"),
        "/word/media/image1.png"
    );
}

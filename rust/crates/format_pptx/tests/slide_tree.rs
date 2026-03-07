use std::fs;
use std::io::Write;

use format_pptx::{parse_part_relationships, parse_slide_tree, PresentationSize};
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
fn parses_presentation_slide_tree_in_document_order() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sample.pptx");
    create_zip(
        &path,
        &[
            (
                "[Content_Types].xml",
                br#"
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml" />
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml" />
  <Override PartName="/ppt/slides/slide2.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml" />
</Types>
"#,
            ),
            (
                "_rels/.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml" />
</Relationships>
"#,
            ),
            (
                "ppt/presentation.xml",
                br#"
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldSz cx="9144000" cy="6858000" />
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rId9" />
  </p:sldMasterIdLst>
  <p:sldIdLst>
    <p:sldId id="256" r:id="rId2" />
    <p:sldId id="512" r:id="rId5" />
  </p:sldIdLst>
</p:presentation>
"#,
            ),
            (
                "ppt/_rels/presentation.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml" />
  <Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide2.xml" />
  <Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml" />
</Relationships>
"#,
            ),
            (
                "ppt/slides/slide1.xml",
                br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld />
  <p:transition advClick="1" />
  <p:timing>
    <p:tnLst>
      <p:par>
        <p:cTn id="1" />
      </p:par>
    </p:tnLst>
  </p:timing>
</p:sld>
"#,
            ),
            (
                "ppt/slides/slide2.xml",
                br#"<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cSld /></p:sld>"#,
            ),
            (
                "ppt/slides/_rels/slide1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml" />
  <Relationship Id="rId8" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide1.xml" />
</Relationships>
"#,
            ),
            (
                "ppt/slides/_rels/slide2.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout2.xml" />
</Relationships>
"#,
            ),
            ("ppt/slideLayouts/slideLayout1.xml", br#"<p:sldLayout xmlns:p="urn:test" />"#),
            ("ppt/slideLayouts/slideLayout2.xml", br#"<p:sldLayout xmlns:p="urn:test" />"#),
            (
                "ppt/slideMasters/slideMaster1.xml",
                br#"<p:sldMaster xmlns:p="urn:test" />"#,
            ),
            (
                "ppt/slideMasters/_rels/slideMaster1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml" />
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout2.xml" />
  <Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml" />
</Relationships>
"#,
            ),
            ("ppt/theme/theme1.xml", br#"<a:theme xmlns:a="urn:test" />"#),
            (
                "ppt/notesSlides/notesSlide1.xml",
                br#"<p:notes xmlns:p="urn:test" />"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let slide_tree = parse_slide_tree(&archive).expect("slide tree should parse");

    assert_eq!(slide_tree.presentation_part, "ppt/presentation.xml");
    assert_eq!(
        slide_tree.presentation_size,
        Some(PresentationSize {
            width_emu: 9_144_000,
            height_emu: 6_858_000,
        })
    );
    assert_eq!(slide_tree.slides.len(), 2);
    assert_eq!(slide_tree.slides[0].slide_id, 256);
    assert_eq!(slide_tree.slides[0].relationship_id, "rId2");
    assert_eq!(slide_tree.slides[0].part_name, "ppt/slides/slide1.xml");
    assert_eq!(
        slide_tree.slides[0].layout_part_name.as_deref(),
        Some("ppt/slideLayouts/slideLayout1.xml")
    );
    assert_eq!(
        slide_tree.slides[0].notes_part_name.as_deref(),
        Some("ppt/notesSlides/notesSlide1.xml")
    );
    assert!(slide_tree.slides[0].has_transition);
    assert_eq!(slide_tree.slides[0].ignored_animation_nodes, 3);
    assert_eq!(slide_tree.slides[1].slide_id, 512);
    assert_eq!(slide_tree.slides[1].part_name, "ppt/slides/slide2.xml");
    assert!(slide_tree.slides[1].notes_part_name.is_none());
    assert!(!slide_tree.slides[1].has_transition);
    assert_eq!(slide_tree.slides[1].ignored_animation_nodes, 0);
    assert_eq!(slide_tree.slide_masters.len(), 1);
    assert_eq!(slide_tree.slide_masters[0].master_id, 2_147_483_648);
    assert_eq!(
        slide_tree.slide_masters[0].theme_part_name.as_deref(),
        Some("ppt/theme/theme1.xml")
    );
    assert_eq!(slide_tree.slide_masters[0].layouts.len(), 2);
    assert_eq!(
        slide_tree.slide_masters[0].layouts[0].part_name,
        "ppt/slideLayouts/slideLayout1.xml"
    );
}

#[test]
fn notes_and_animation_metadata_do_not_block_slide_tree_parse() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("notes-and-animations.pptx");
    create_zip(
        &path,
        &[
            (
                "[Content_Types].xml",
                br#"
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Override PartName="/ppt/presentation.xml" ContentType="application/test" />
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/test" />
  <Override PartName="/ppt/notesSlides/notesSlide1.xml" ContentType="application/test" />
</Types>
"#,
            ),
            (
                "_rels/.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml" />
</Relationships>
"#,
            ),
            (
                "ppt/presentation.xml",
                br#"
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldIdLst>
    <p:sldId id="256" r:id="rId2" />
  </p:sldIdLst>
</p:presentation>
"#,
            ),
            (
                "ppt/_rels/presentation.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml" />
</Relationships>
"#,
            ),
            (
                "ppt/slides/slide1.xml",
                br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld />
  <p:timing>
    <p:bldLst>
      <p:bldP />
    </p:bldLst>
  </p:timing>
</p:sld>
"#,
            ),
            (
                "ppt/slides/_rels/slide1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide1.xml" />
</Relationships>
"#,
            ),
            (
                "ppt/notesSlides/notesSlide1.xml",
                br#"<p:notes xmlns:p="urn:test" />"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let slide_tree = parse_slide_tree(&archive).expect("slide tree should parse");

    assert_eq!(slide_tree.slides.len(), 1);
    assert_eq!(
        slide_tree.slides[0].notes_part_name.as_deref(),
        Some("ppt/notesSlides/notesSlide1.xml")
    );
    assert_eq!(slide_tree.slides[0].ignored_animation_nodes, 2);
}

#[test]
fn missing_slide_relationship_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("missing-slide-rel.pptx");
    create_zip(
        &path,
        &[
            (
                "[Content_Types].xml",
                br#"
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Override PartName="/ppt/presentation.xml" ContentType="application/test" />
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/test" />
</Types>
"#,
            ),
            (
                "_rels/.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml" />
</Relationships>
"#,
            ),
            (
                "ppt/presentation.xml",
                br#"
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldIdLst>
    <p:sldId id="256" r:id="rId9" />
  </p:sldIdLst>
</p:presentation>
"#,
            ),
            (
                "ppt/_rels/presentation.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml" />
</Relationships>
"#,
            ),
            ("ppt/slides/slide1.xml", br#"<p:sld xmlns:p="urn:test" />"#),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let error = parse_slide_tree(&archive).expect_err("missing slide rel should fail");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

#[test]
fn missing_slide_master_relationship_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("missing-master-rel.pptx");
    create_zip(
        &path,
        &[
            (
                "[Content_Types].xml",
                br#"
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Override PartName="/ppt/presentation.xml" ContentType="application/test" />
</Types>
"#,
            ),
            (
                "_rels/.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml" />
</Relationships>
"#,
            ),
            (
                "ppt/presentation.xml",
                br#"
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rId77" />
  </p:sldMasterIdLst>
</p:presentation>
"#,
            ),
            (
                "ppt/_rels/presentation.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml" />
</Relationships>
"#,
            ),
            (
                "ppt/slideMasters/slideMaster1.xml",
                br#"<p:sldMaster xmlns:p=\"urn:test\" />"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let error = parse_slide_tree(&archive).expect_err("missing master rel should fail");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

#[test]
fn parses_presentation_relationships_and_allows_external_targets() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("presentation-rels.pptx");
    create_zip(
        &path,
        &[
            ("ppt/presentation.xml", br#"<p:presentation xmlns:p="urn:test" />"#),
            (
                "ppt/_rels/presentation.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml" />
  <Relationship Id="rId7" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com" TargetMode="External" />
</Relationships>
"#,
            ),
            ("ppt/slides/slide1.xml", br#"<p:sld xmlns:p="urn:test" />"#),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let relationships = parse_part_relationships(&archive, "ppt/presentation.xml")
        .expect("presentation rels should parse");

    assert_eq!(relationships.len(), 2);
    assert_eq!(relationships[0].resolved_target, "/ppt/slides/slide1.xml");
    assert_eq!(relationships[1].resolved_target, "https://example.com");
}

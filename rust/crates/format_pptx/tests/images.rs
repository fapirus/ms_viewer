use std::fs;
use std::io::Write;

use format_pptx::parse_slide_images;
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
fn parses_embedded_and_external_slide_images() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("images.pptx");
    create_zip(
        &path,
        &[
            (
                "ppt/slides/slide1.xml",
                br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:pic>
        <p:nvPicPr>
          <p:cNvPr id="2" name="Logo" descr="Brand logo" />
          <p:cNvPicPr />
          <p:nvPr />
        </p:nvPicPr>
        <p:blipFill>
          <a:blip r:embed="rId3" />
          <a:srcRect l="1000" t="2000" r="3000" b="4000" />
        </p:blipFill>
        <p:spPr>
          <a:xfrm>
            <a:off x="100" y="200" />
            <a:ext cx="300" cy="400" />
          </a:xfrm>
        </p:spPr>
      </p:pic>
      <p:pic>
        <p:nvPicPr>
          <p:cNvPr id="3" name="Linked Image" />
          <p:cNvPicPr />
          <p:nvPr />
        </p:nvPicPr>
        <p:blipFill>
          <a:blip r:link="rId9" />
        </p:blipFill>
        <p:spPr>
          <a:xfrm>
            <a:off x="500" y="600" />
            <a:ext cx="700" cy="800" />
          </a:xfrm>
        </p:spPr>
      </p:pic>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
            ),
            (
                "ppt/slides/_rels/slide1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png" />
  <Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="https://example.com/photo.jpeg" TargetMode="External" />
</Relationships>
"#,
            ),
            ("ppt/media/image1.png", b"fakepng"),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let images =
        parse_slide_images(&archive, "ppt/slides/slide1.xml").expect("images should parse");

    assert_eq!(images.len(), 2);

    let embedded = &images[0];
    assert_eq!(embedded.shape_id, 2);
    assert_eq!(embedded.name, "Logo");
    assert_eq!(embedded.description.as_deref(), Some("Brand logo"));
    assert_eq!(embedded.image.resource_id, "ppt/media/image1.png");
    assert_eq!(embedded.image.content_type.as_deref(), Some("image/png"));
    assert_eq!(embedded.image.display_width, Some(300.0));
    assert_eq!(embedded.image.display_height, Some(400.0));
    assert!(!embedded.is_external);
    let crop = embedded.crop.as_ref().expect("crop");
    assert_eq!(crop.left, 1000);
    assert_eq!(crop.bottom, 4000);

    let external = &images[1];
    assert_eq!(external.shape_id, 3);
    assert_eq!(external.image.resource_id, "https://example.com/photo.jpeg");
    assert_eq!(external.image.content_type.as_deref(), Some("image/jpeg"));
    assert!(external.is_external);
    assert_eq!(external.description.as_deref(), Some("Linked Image"));
    assert!(external.crop.is_none());
}

#[test]
fn missing_image_relationship_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("missing-image-rel.pptx");
    create_zip(
        &path,
        &[
            (
                "ppt/slides/slide1.xml",
                br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:pic>
        <p:nvPicPr>
          <p:cNvPr id="2" name="Missing" />
          <p:cNvPicPr />
          <p:nvPr />
        </p:nvPicPr>
        <p:blipFill>
          <a:blip r:embed="rId404" />
        </p:blipFill>
      </p:pic>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
            ),
            (
                "ppt/slides/_rels/slide1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png" />
</Relationships>
"#,
            ),
            ("ppt/media/image1.png", b"fakepng"),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let error =
        parse_slide_images(&archive, "ppt/slides/slide1.xml").expect_err("missing image rel");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

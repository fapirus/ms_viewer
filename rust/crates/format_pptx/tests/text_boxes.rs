use std::fs;
use std::io::Write;

use format_pptx::{
    parse_slide_text_boxes, SlidePlaceholderKind, SlideTextAlignment,
};
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
fn parses_slide_text_boxes_with_placeholder_bounds_and_runs() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("text-boxes.pptx");
    create_zip(
        &path,
        &[(
            "ppt/slides/slide1.xml",
            br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Title 1" />
          <p:cNvSpPr />
          <p:nvPr>
            <p:ph type="title" />
          </p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="457200" y="228600" />
            <a:ext cx="8229600" cy="1143000" />
          </a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr />
          <a:lstStyle />
          <a:p>
            <a:pPr algn="ctr" />
            <a:r>
              <a:rPr b="1" i="1" sz="2400">
                <a:latin typeface="Aptos" />
                <a:ea typeface="Malgun Gothic" />
                <a:solidFill>
                  <a:srgbClr val="112233" />
                </a:solidFill>
              </a:rPr>
              <a:t>Hello</a:t>
            </a:r>
            <a:br />
            <a:r>
              <a:t>World</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Body Placeholder 2" />
          <p:cNvSpPr />
          <p:nvPr>
            <p:ph type="body" />
          </p:nvPr>
        </p:nvSpPr>
        <p:txBody>
          <a:bodyPr />
          <a:lstStyle />
          <a:p>
            <a:pPr lvl="1" />
            <a:fld id="{00000000-0000-0000-0000-000000000000}" type="slidenum">
              <a:rPr sz="1800" />
              <a:t>42</a:t>
            </a:fld>
          </a:p>
        </p:txBody>
      </p:sp>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="4" name="Decorative Shape" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr />
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
        )],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let text_boxes =
        parse_slide_text_boxes(&archive, "ppt/slides/slide1.xml").expect("text boxes should parse");

    assert_eq!(text_boxes.len(), 2);

    let title = &text_boxes[0];
    assert_eq!(title.shape_id, 2);
    assert_eq!(title.name, "Title 1");
    assert_eq!(title.placeholder, Some(SlidePlaceholderKind::Title));
    assert_eq!(title.bounds.as_ref().expect("bounds").x, 457_200);
    assert_eq!(title.bounds.as_ref().expect("bounds").width, 8_229_600);
    assert_eq!(title.paragraphs.len(), 1);
    assert_eq!(title.paragraphs[0].alignment, Some(SlideTextAlignment::Center));
    assert_eq!(title.paragraphs[0].runs.len(), 3);
    assert_eq!(title.paragraphs[0].runs[0].text, "Hello");
    assert!(title.paragraphs[0].runs[0].style.bold);
    assert!(title.paragraphs[0].runs[0].style.italic);
    assert_eq!(
        title.paragraphs[0].runs[0].style.font_face.as_deref(),
        Some("Aptos")
    );
    assert_eq!(
        title.paragraphs[0].runs[0]
            .style
            .east_asia_font_face
            .as_deref(),
        Some("Malgun Gothic")
    );
    assert_eq!(
        title.paragraphs[0].runs[0].style.font_size_centipoints,
        Some(2400)
    );
    assert_eq!(title.paragraphs[0].runs[0].style.color.as_deref(), Some("#112233"));
    assert_eq!(title.paragraphs[0].runs[1].text, "\n");
    assert_eq!(title.paragraphs[0].runs[2].text, "World");

    let body = &text_boxes[1];
    assert_eq!(body.placeholder, Some(SlidePlaceholderKind::Body));
    assert!(body.bounds.is_none());
    assert_eq!(body.paragraphs[0].level, Some(1));
    assert_eq!(body.paragraphs[0].runs.len(), 1);
    assert_eq!(body.paragraphs[0].runs[0].text, "42");
    assert_eq!(
        body.paragraphs[0].runs[0].style.font_size_centipoints,
        Some(1800)
    );
}

#[test]
fn invalid_transform_geometry_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("invalid-transform.pptx");
    create_zip(
        &path,
        &[(
            "ppt/slides/slide1.xml",
            br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Title 1" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="1" y="2" />
          </a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr />
          <a:lstStyle />
          <a:p>
            <a:r><a:t>Hello</a:t></a:r>
          </a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
        )],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let error =
        parse_slide_text_boxes(&archive, "ppt/slides/slide1.xml").expect_err("invalid xfrm");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

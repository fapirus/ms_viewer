use std::fs;
use std::io::Write;

use format_pptx::{parse_slide_basic_shapes, BasicShapeGeometry, ShapeFill};
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
fn parses_basic_shapes_with_geometry_fill_stroke_and_transform() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("basic-shapes.pptx");
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
          <p:cNvPr id="2" name="Rounded Rectangle 1" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm rot="5400000" flipH="1">
            <a:off x="1000" y="2000" />
            <a:ext cx="3000" cy="4000" />
          </a:xfrm>
          <a:prstGeom prst="roundRect">
            <a:avLst />
          </a:prstGeom>
          <a:solidFill>
            <a:srgbClr val="FF0000" />
          </a:solidFill>
          <a:ln w="12700">
            <a:solidFill>
              <a:schemeClr val="accent1" />
            </a:solidFill>
          </a:ln>
        </p:spPr>
        <p:txBody>
          <a:bodyPr />
          <a:lstStyle />
          <a:p><a:r><a:t>Shape text</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Ellipse 2" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm flipV="1">
            <a:off x="5000" y="6000" />
            <a:ext cx="7000" cy="8000" />
          </a:xfrm>
          <a:prstGeom prst="ellipse">
            <a:avLst />
          </a:prstGeom>
          <a:noFill />
          <a:ln w="9525">
            <a:noFill />
          </a:ln>
        </p:spPr>
      </p:sp>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="4" name="Title Only" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:txBody>
          <a:bodyPr />
          <a:lstStyle />
          <a:p><a:r><a:t>Ignored text box</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
        )],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let shapes =
        parse_slide_basic_shapes(&archive, "ppt/slides/slide1.xml").expect("shapes should parse");

    assert_eq!(shapes.len(), 2);

    let first = &shapes[0];
    assert_eq!(first.shape_id, 2);
    assert_eq!(first.name, "Rounded Rectangle 1");
    assert_eq!(
        first.geometry,
        BasicShapeGeometry::Preset("roundRect".to_string())
    );
    assert!(first.has_text_body);
    let transform = first.transform.as_ref().expect("transform");
    assert_eq!(transform.bounds.x, 1000);
    assert_eq!(transform.bounds.height, 4000);
    assert_eq!(transform.rotation_units, Some(5_400_000));
    assert!(transform.flip_horizontal);
    assert!(!transform.flip_vertical);
    assert_eq!(first.fill, Some(ShapeFill::Solid("#FF0000".to_string())));
    let stroke = first.stroke.as_ref().expect("stroke");
    assert_eq!(stroke.width_emu, Some(12_700));
    assert_eq!(stroke.color.as_deref(), Some("scheme:accent1"));
    assert!(!stroke.is_none);

    let second = &shapes[1];
    assert_eq!(second.shape_id, 3);
    assert_eq!(
        second.geometry,
        BasicShapeGeometry::Preset("ellipse".to_string())
    );
    assert_eq!(second.fill, Some(ShapeFill::None));
    assert!(second.transform.as_ref().expect("transform").flip_vertical);
    let stroke = second.stroke.as_ref().expect("stroke");
    assert_eq!(stroke.width_emu, Some(9_525));
    assert!(stroke.is_none);
    assert!(stroke.color.is_none());
}

#[test]
fn invalid_shape_stroke_color_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("invalid-shape-stroke.pptx");
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
          <p:cNvPr id="2" name="Rectangle 1" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr>
          <a:prstGeom prst="rect">
            <a:avLst />
          </a:prstGeom>
          <a:ln w="12700">
            <a:solidFill />
          </a:ln>
        </p:spPr>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
        )],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let error = parse_slide_basic_shapes(&archive, "ppt/slides/slide1.xml")
        .expect_err("invalid stroke color should fail");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

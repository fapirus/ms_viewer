use std::fs;
use std::io::Write;

use format_pptx::{build_slide_render_model, PptxSlideTree, PresentationSize, SlideReference};
use tempfile::tempdir;
use viewer_core::archive::OoxmlArchive;
use viewer_core::model::RenderNode;
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
fn builds_slide_render_model_from_shapes_images_and_text() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("render-model.pptx");
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
            <a:off x="127000" y="254000" />
            <a:ext cx="3048000" cy="914400" />
          </a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr />
          <a:lstStyle />
          <a:p>
            <a:pPr algn="ctr" />
            <a:r>
              <a:rPr sz="2400" b="1">
                <a:latin typeface="Aptos" />
              </a:rPr>
              <a:t>Title Text</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Rectangle 2" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="508000" y="1524000" />
            <a:ext cx="1524000" cy="762000" />
          </a:xfrm>
          <a:prstGeom prst="rect"><a:avLst /></a:prstGeom>
          <a:solidFill><a:srgbClr val="FFAA00" /></a:solidFill>
          <a:ln w="12700"><a:solidFill><a:srgbClr val="333333" /></a:solidFill></a:ln>
        </p:spPr>
      </p:sp>
      <p:pic>
        <p:nvPicPr>
          <p:cNvPr id="4" name="Picture 3" descr="Sample image" />
          <p:cNvPicPr />
          <p:nvPr />
        </p:nvPicPr>
        <p:blipFill>
          <a:blip r:embed="rIdImage1" />
        </p:blipFill>
        <p:spPr>
          <a:xfrm>
            <a:off x="3810000" y="1778000" />
            <a:ext cx="1270000" cy="952500" />
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
  <Relationship Id="rIdImage1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png" />
</Relationships>
"#,
            ),
            ("ppt/media/image1.png", b"fake-png"),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let slide_tree = PptxSlideTree {
        presentation_part: "ppt/presentation.xml".to_string(),
        presentation_size: Some(PresentationSize {
            width_emu: 9_144_000,
            height_emu: 6_858_000,
        }),
        slides: vec![SlideReference {
            slide_id: 256,
            relationship_id: "rIdSlide1".to_string(),
            part_name: "ppt/slides/slide1.xml".to_string(),
            layout_part_name: None,
            notes_part_name: None,
            has_transition: false,
            ignored_animation_nodes: 0,
        }],
        slide_masters: Vec::new(),
    };
    let page = build_slide_render_model(&archive, &slide_tree, 0).expect("render model");

    assert_eq!(page.page_index, 0);
    assert_eq!(page.width, 720.0);
    assert_eq!(page.height, 540.0);
    assert!(page.selection_anchors.is_empty());

    let text_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(node) => Some(node),
            _ => None,
        })
        .collect();
    assert_eq!(text_nodes.len(), 1);
    assert_eq!(text_nodes[0].text, "Title Text");
    assert_eq!(text_nodes[0].style.font_family, "Aptos");
    assert_eq!(text_nodes[0].style.font_size, 24.0);
    assert!(text_nodes[0].bounds.x > 10.0);

    let box_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Box(node) => Some(node),
            _ => None,
        })
        .collect();
    assert_eq!(box_nodes.len(), 1);
    assert_eq!(box_nodes[0].fill_color_hex.as_deref(), Some("#FFAA00"));
    assert_eq!(box_nodes[0].stroke_color_hex.as_deref(), Some("#333333"));

    let image_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Image(node) => Some(node),
            _ => None,
        })
        .collect();
    assert_eq!(image_nodes.len(), 1);
    assert_eq!(image_nodes[0].resource_id, "ppt/media/image1.png");
    assert_eq!(image_nodes[0].description.as_deref(), Some("Sample image"));
    assert!(image_nodes[0].data_base64.is_some());
}

#[test]
fn render_model_ignores_non_hex_shape_colors_but_keeps_nodes() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("render-model-colors.pptx");
    create_zip(
        &path,
        &[
            (
                "ppt/slides/slide1.xml",
                br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Scheme Rectangle" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="0" y="0" />
            <a:ext cx="1270000" cy="1270000" />
          </a:xfrm>
          <a:prstGeom prst="rect"><a:avLst /></a:prstGeom>
          <a:solidFill><a:schemeClr val="accent1" /></a:solidFill>
        </p:spPr>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
            ),
            (
                "ppt/slides/_rels/slide1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships" />
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let slide_tree = PptxSlideTree {
        presentation_part: "ppt/presentation.xml".to_string(),
        presentation_size: None,
        slides: vec![SlideReference {
            slide_id: 256,
            relationship_id: "rIdSlide1".to_string(),
            part_name: "ppt/slides/slide1.xml".to_string(),
            layout_part_name: None,
            notes_part_name: None,
            has_transition: false,
            ignored_animation_nodes: 0,
        }],
        slide_masters: Vec::new(),
    };
    let page = build_slide_render_model(&archive, &slide_tree, 0).expect("render model");

    let box_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Box(node) => Some(node),
            _ => None,
        })
        .collect();
    assert_eq!(box_nodes.len(), 1);
    assert!(box_nodes[0].fill_color_hex.is_none());
}

use std::fs;
use std::io::Write;

use format_pptx::{
    build_slide_render_model, parse_pptx, PptxSlideTree, PresentationSize, SlideReference,
};
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
    assert_eq!(page.selection_anchors.len(), "Title Text".chars().count() + 1);

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
    assert!(text_nodes[0].bounds.x > 60.0);
    assert!(text_nodes[0].bounds.x < 70.0);
    assert_eq!(text_nodes[0].range.start, 0);
    assert_eq!(text_nodes[0].range.end, 10);

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
fn wraps_text_into_multiple_lines_with_level_indent_inside_box() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("render-model-wrap.pptx");
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
          <p:cNvPr id="2" name="Body 1" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="254000" y="254000" />
            <a:ext cx="1200000" cy="2000000" />
          </a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr lIns="127000" rIns="127000" tIns="127000" bIns="127000" />
          <a:lstStyle />
          <a:p>
            <a:pPr lvl="1" />
            <a:r>
              <a:rPr sz="2000">
                <a:latin typeface="Aptos" />
              </a:rPr>
              <a:t>alpha beta gamma delta</a:t>
            </a:r>
          </a:p>
        </p:txBody>
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

    let text_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(node) => Some(node),
            _ => None,
        })
        .collect();

    assert!(text_nodes.len() >= 3);
    assert!(!page.selection_anchors.is_empty());
    assert_eq!(
        page.selection_anchors.last().expect("last anchor").char_index,
        text_nodes
            .last()
            .expect("last text node")
            .text
            .chars()
            .count() as u32
    );
    let mut line_ys: Vec<i32> = text_nodes
        .iter()
        .map(|node| node.bounds.y.round() as i32)
        .collect();
    line_ys.sort_unstable();
    line_ys.dedup();
    assert!(line_ys.len() >= 3);
    assert!(text_nodes.iter().all(|node| node.bounds.x >= 48.0));
    assert!(text_nodes
        .iter()
        .all(|node| node.bounds.x + node.bounds.width <= 114.6));
    assert_eq!(text_nodes.first().expect("first").range.start, 0);
    assert!(text_nodes
        .windows(2)
        .all(|pair| pair[0].range.end == pair[1].range.start));
}

#[test]
fn applies_layout_text_defaults_theme_fonts_and_bullets() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("render-model-theme-text.pptx");
    create_zip(
        &path,
        &[
            (
                "[Content_Types].xml",
                br#"
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
</Types>
"#,
            ),
            (
                "_rels/.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>
"#,
            ),
            (
                "ppt/presentation.xml",
                br#"
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rIdMaster1"/></p:sldMasterIdLst>
  <p:sldIdLst><p:sldId id="256" r:id="rIdSlide1"/></p:sldIdLst>
  <p:sldSz cx="9144000" cy="6858000"/>
</p:presentation>
"#,
            ),
            (
                "ppt/_rels/presentation.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdMaster1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
  <Relationship Id="rIdSlide1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
</Relationships>
"#,
            ),
            (
                "ppt/slides/slide1.xml",
                br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="2" name="Title"/><p:cNvSpPr/><p:nvPr><p:ph type="ctrTitle"/></p:nvPr></p:nvSpPr>
        <p:spPr><a:xfrm><a:off x="381000" y="381000"/><a:ext cx="5334000" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
        <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>Theme Title</a:t></a:r></a:p></p:txBody>
      </p:sp>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="3" name="Body"/><p:cNvSpPr/><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr>
        <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>Bullet body</a:t></a:r></a:p></p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
            ),
            (
                "ppt/slides/_rels/slide1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdLayout1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>
"#,
            ),
            (
                "ppt/slideLayouts/slideLayout1.xml",
                r#"
<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="2" name="Layout Title"/><p:cNvSpPr/><p:nvPr><p:ph type="ctrTitle"/></p:nvPr></p:nvSpPr>
        <p:spPr><a:xfrm><a:off x="381000" y="381000"/><a:ext cx="5334000" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle>
            <a:lvl1pPr>
              <a:defRPr sz="4800" b="1">
                <a:gradFill>
                  <a:gsLst>
                    <a:gs pos="0"><a:srgbClr val="003EA7"/></a:gs>
                    <a:gs pos="100000"><a:srgbClr val="70AD47"/></a:gs>
                  </a:gsLst>
                  <a:lin ang="1920000" scaled="0"/>
                </a:gradFill>
                <a:latin typeface="+mn-lt"/>
                <a:ea typeface="+mn-ea"/>
              </a:defRPr>
            </a:lvl1pPr>
          </a:lstStyle>
          <a:p/>
        </p:txBody>
      </p:sp>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="3" name="Layout Body"/><p:cNvSpPr/><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr>
        <p:spPr><a:xfrm><a:off x="381000" y="1524000"/><a:ext cx="5334000" cy="1016000"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
        <p:txBody>
          <a:bodyPr lIns="127000" rIns="127000"/>
          <a:lstStyle>
            <a:lvl1pPr marL="342900" indent="-342900">
              <a:buClr><a:srgbClr val="0070C0"/></a:buClr>
              <a:buFont typeface="Wingdings"/>
              <a:buChar char="§"/>
              <a:defRPr sz="1800">
                <a:solidFill><a:schemeClr val="tx1"><a:lumMod val="85000"/><a:lumOff val="15000"/></a:schemeClr></a:solidFill>
                <a:latin typeface="+mn-lt"/>
                <a:ea typeface="+mn-ea"/>
              </a:defRPr>
            </a:lvl1pPr>
          </a:lstStyle>
          <a:p/>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sldLayout>
"#
                .as_bytes(),
            ),
            (
                "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdMaster1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>
"#,
            ),
            (
                "ppt/slideMasters/slideMaster1.xml",
                br#"
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
    </p:spTree>
  </p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
</p:sldMaster>
"#,
            ),
            (
                "ppt/slideMasters/_rels/slideMaster1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdLayout1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rIdTheme1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>
"#,
            ),
            (
                "ppt/theme/theme1.xml",
                r#"
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Fixture Theme">
  <a:themeElements>
    <a:clrScheme name="Fixture">
      <a:dk1><a:srgbClr val="111111"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="222222"/></a:dk2>
      <a:lt2><a:srgbClr val="EEEEEE"/></a:lt2>
      <a:accent1><a:srgbClr val="2F45A5"/></a:accent1>
      <a:accent2><a:srgbClr val="4BB0D8"/></a:accent2>
      <a:accent3><a:srgbClr val="70AD47"/></a:accent3>
      <a:accent4><a:srgbClr val="FFC000"/></a:accent4>
      <a:accent5><a:srgbClr val="5B9BD5"/></a:accent5>
      <a:accent6><a:srgbClr val="7030A0"/></a:accent6>
      <a:hlink><a:srgbClr val="0563C1"/></a:hlink>
      <a:folHlink><a:srgbClr val="954F72"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="Fixture Fonts">
      <a:majorFont>
        <a:latin typeface="맑은 고딕"/>
        <a:ea typeface=""/>
        <a:cs typeface=""/>
        <a:font script="Hang" typeface="맑은 고딕"/>
      </a:majorFont>
      <a:minorFont>
        <a:latin typeface="맑은 고딕"/>
        <a:ea typeface=""/>
        <a:cs typeface=""/>
        <a:font script="Hang" typeface="맑은 고딕"/>
      </a:minorFont>
    </a:fontScheme>
  </a:themeElements>
</a:theme>
"#
                .as_bytes(),
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let slide_tree = parse_pptx(&archive).expect("slide tree");
    let page = build_slide_render_model(&archive, &slide_tree, 0).expect("render model");

    let text_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(node) => Some(node),
            _ => None,
        })
        .collect();

    let title = text_nodes
        .iter()
        .find(|node| node.text == "Theme Title")
        .expect("title text node");
    assert_eq!(title.style.font_family, "맑은 고딕");
    assert_eq!(title.style.font_size, 48.0);
    assert_eq!(title.style.color_hex, "#003EA7");
    assert_eq!(title.style.gradient_end_color_hex.as_deref(), Some("#70AD47"));

    let bullet = text_nodes
        .iter()
        .find(|node| node.text == "▪")
        .expect("bullet node");
    assert_eq!(bullet.style.color_hex, "#0070C0");

    let body = text_nodes
        .iter()
        .find(|node| node.text == "Bullet body")
        .expect("body text node");
    assert_eq!(body.style.font_family, "맑은 고딕");
    assert_eq!(body.style.font_size, 18.0);
    assert!(body.bounds.x > bullet.bounds.x);
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

#[test]
fn render_model_inherits_layout_bounds_recurses_groups_and_renders_tables_and_background() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("render-model-inheritance.pptx");
    create_zip(
        &path,
        &[
            (
                "[Content_Types].xml",
                br#"
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="png" ContentType="image/png"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
</Types>
"#,
            ),
            (
                "_rels/.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>
"#,
            ),
            (
                "ppt/presentation.xml",
                br#"
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rIdMaster1"/>
  </p:sldMasterIdLst>
  <p:sldIdLst>
    <p:sldId id="256" r:id="rIdSlide1"/>
  </p:sldIdLst>
  <p:sldSz cx="9144000" cy="6858000"/>
</p:presentation>
"#,
            ),
            (
                "ppt/_rels/presentation.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdMaster1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
  <Relationship Id="rIdSlide1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
</Relationships>
"#,
            ),
            (
                "ppt/slides/slide1.xml",
                br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Body Placeholder"/>
          <p:cNvSpPr/>
          <p:nvPr><p:ph type="body" idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p><a:r><a:t>Inherited placeholder</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
      <p:grpSp>
        <p:nvGrpSpPr><p:cNvPr id="3" name="Group 1"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
        <p:grpSpPr>
          <a:xfrm>
            <a:off x="127000" y="1778000"/>
            <a:ext cx="2540000" cy="1524000"/>
            <a:chOff x="0" y="0"/>
            <a:chExt cx="2540000" cy="1524000"/>
          </a:xfrm>
        </p:grpSpPr>
        <p:pic>
          <p:nvPicPr><p:cNvPr id="4" name="Grouped Picture"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr>
          <p:blipFill><a:blip r:embed="rIdImage1"/></p:blipFill>
          <p:spPr>
            <a:xfrm><a:off x="254000" y="254000"/><a:ext cx="1270000" cy="762000"/></a:xfrm>
            <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
          </p:spPr>
        </p:pic>
      </p:grpSp>
      <p:graphicFrame>
        <p:nvGraphicFramePr><p:cNvPr id="5" name="Table 1"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr>
        <p:xfrm><a:off x="5080000" y="1778000"/><a:ext cx="2032000" cy="1016000"/></p:xfrm>
        <a:graphic>
          <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table">
            <a:tbl>
              <a:tblGrid>
                <a:gridCol w="1016000"/>
                <a:gridCol w="1016000"/>
              </a:tblGrid>
              <a:tr h="508000">
                <a:tc>
                  <a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>Cell A</a:t></a:r></a:p></a:txBody>
                  <a:tcPr marL="63500" marR="63500" marT="31750" marB="31750"><a:solidFill><a:schemeClr val="accent1"/></a:solidFill></a:tcPr>
                </a:tc>
                <a:tc>
                  <a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>Cell B</a:t></a:r></a:p></a:txBody>
                  <a:tcPr marL="63500" marR="63500" marT="31750" marB="31750"><a:lnL w="12700"><a:solidFill><a:schemeClr val="accent2"/></a:solidFill></a:lnL></a:tcPr>
                </a:tc>
              </a:tr>
            </a:tbl>
          </a:graphicData>
        </a:graphic>
      </p:graphicFrame>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
            ),
            (
                "ppt/slides/_rels/slide1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdLayout1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rIdImage1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png"/>
</Relationships>
"#,
            ),
            (
                "ppt/slideLayouts/slideLayout1.xml",
                br#"
<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Body Placeholder Layout"/>
          <p:cNvSpPr/>
          <p:nvPr><p:ph type="body" idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="762000" y="889000"/><a:ext cx="3810000" cy="1016000"/></a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
        </p:spPr>
        <p:txBody><a:bodyPr lIns="127000" rIns="127000" tIns="63500" bIns="63500"/><a:lstStyle/><a:p/></p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sldLayout>
"#,
            ),
            (
                "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdMaster1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>
"#,
            ),
            (
                "ppt/slideMasters/slideMaster1.xml",
                br#"
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="2" name="Background"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="0" y="0"/><a:ext cx="9144000" cy="6858000"/></a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
          <a:gradFill>
            <a:gsLst>
              <a:gs pos="0"><a:srgbClr val="003EA7"/></a:gs>
              <a:gs pos="100000"><a:srgbClr val="70AD47"/></a:gs>
            </a:gsLst>
            <a:lin ang="3720000" scaled="0"/>
          </a:gradFill>
          <a:ln><a:noFill/></a:ln>
        </p:spPr>
      </p:sp>
    </p:spTree>
  </p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
</p:sldMaster>
"#,
            ),
            (
                "ppt/slideMasters/_rels/slideMaster1.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdLayout1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rIdTheme1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>
"#,
            ),
            (
                "ppt/theme/theme1.xml",
                br#"
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Fixture Theme">
  <a:themeElements>
    <a:clrScheme name="Fixture">
      <a:dk1><a:srgbClr val="111111"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="222222"/></a:dk2>
      <a:lt2><a:srgbClr val="EEEEEE"/></a:lt2>
      <a:accent1><a:srgbClr val="2F45A5"/></a:accent1>
      <a:accent2><a:srgbClr val="4BB0D8"/></a:accent2>
      <a:accent3><a:srgbClr val="70AD47"/></a:accent3>
      <a:accent4><a:srgbClr val="FFC000"/></a:accent4>
      <a:accent5><a:srgbClr val="5B9BD5"/></a:accent5>
      <a:accent6><a:srgbClr val="7030A0"/></a:accent6>
      <a:hlink><a:srgbClr val="0563C1"/></a:hlink>
      <a:folHlink><a:srgbClr val="954F72"/></a:folHlink>
    </a:clrScheme>
  </a:themeElements>
</a:theme>
"#,
            ),
            ("ppt/media/image1.png", b"fake-png"),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let slide_tree = parse_pptx(&archive).expect("slide tree");
    let page = build_slide_render_model(&archive, &slide_tree, 0).expect("render model");

    let text_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(node) => Some(node),
            _ => None,
        })
        .collect();
    let box_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Box(node) => Some(node),
            _ => None,
        })
        .collect();
    let image_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Image(node) => Some(node),
            _ => None,
        })
        .collect();

    assert!(text_nodes.iter().any(|node| node.text.contains("Inherited placeholder")));
    assert!(text_nodes.iter().any(|node| node.text.contains("Cell A")));
    assert!(text_nodes.iter().any(|node| node.text.contains("Cell B")));
    assert_eq!(image_nodes.len(), 1);
    assert!(image_nodes[0].bounds.x > 25.0);
    assert!(box_nodes.iter().any(|node| {
        node.bounds.width >= 719.0
            && (node.fill_color_hex.is_some() || node.gradient_end_color_hex.is_some())
    }));
    assert!(box_nodes.iter().any(|node| node.fill_color_hex.as_deref() == Some("#2F45A5")));
    assert!(box_nodes.iter().any(|node| node.stroke_color_hex.as_deref() == Some("#4BB0D8")));
}

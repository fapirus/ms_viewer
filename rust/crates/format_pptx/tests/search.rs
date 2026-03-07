use std::fs;
use std::io::Write;

use format_pptx::{build_search_pages, search_slides, PptxSlideTree, SlideReference};
use tempfile::tempdir;
use viewer_core::archive::OoxmlArchive;
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

fn sample_slide_tree() -> PptxSlideTree {
    PptxSlideTree {
        presentation_part: "ppt/presentation.xml".to_string(),
        presentation_size: None,
        slides: vec![
            SlideReference {
                slide_id: 256,
                relationship_id: "rId1".to_string(),
                part_name: "ppt/slides/slide1.xml".to_string(),
                layout_part_name: None,
                notes_part_name: None,
                has_transition: false,
                ignored_animation_nodes: 0,
            },
            SlideReference {
                slide_id: 257,
                relationship_id: "rId2".to_string(),
                part_name: "ppt/slides/slide2.xml".to_string(),
                layout_part_name: None,
                notes_part_name: None,
                has_transition: false,
                ignored_animation_nodes: 0,
            },
        ],
        slide_masters: Vec::new(),
    }
}

#[test]
fn builds_search_pages_from_slide_text_boxes() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("search-pages.pptx");
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
          <p:cNvPr id="2" name="Title 1" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr />
        <p:txBody>
          <a:bodyPr />
          <a:lstStyle />
          <a:p>
            <a:r><a:t>Quarterly Results</a:t></a:r>
          </a:p>
          <a:p>
            <a:r><a:t>Revenue grew 20%</a:t></a:r>
          </a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
            ),
            (
                "ppt/slides/slide2.xml",
                br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Body 2" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr />
        <p:txBody>
          <a:bodyPr />
          <a:lstStyle />
          <a:p>
            <a:r><a:t>Forecast</a:t></a:r>
            <a:br />
            <a:r><a:t>North America</a:t></a:r>
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
                br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships" />"#,
            ),
            (
                "ppt/slides/_rels/slide2.xml.rels",
                br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships" />"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let pages = build_search_pages(&archive, &sample_slide_tree()).expect("search pages");

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].page_index, 0);
    assert_eq!(pages[0].text, "Quarterly Results\nRevenue grew 20%");
    assert_eq!(pages[1].page_index, 1);
    assert_eq!(pages[1].text, "Forecast\nNorth America");
}

#[test]
fn slide_search_is_case_insensitive_and_slide_scoped() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("search-matches.pptx");
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
          <p:cNvPr id="2" name="Slide One" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr />
        <p:txBody>
          <a:bodyPr />
          <a:lstStyle />
          <a:p>
            <a:r><a:t>Revenue revenue</a:t></a:r>
          </a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>
"#,
            ),
            (
                "ppt/slides/slide2.xml",
                br#"
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Slide Two" />
          <p:cNvSpPr />
          <p:nvPr />
        </p:nvSpPr>
        <p:spPr />
        <p:txBody>
          <a:bodyPr />
          <a:lstStyle />
          <a:p>
            <a:r><a:t>No match here</a:t></a:r>
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
                br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships" />"#,
            ),
            (
                "ppt/slides/_rels/slide2.xml.rels",
                br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships" />"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let matches = search_slides(&archive, &sample_slide_tree(), "REVENUE").expect("matches");

    assert_eq!(matches.len(), 2);
    assert!(matches.iter().all(|item| item.page_index == 0));
    assert_eq!(matches[0].query, "REVENUE");
    assert!(matches[0].preview.to_lowercase().contains("revenue"));
}

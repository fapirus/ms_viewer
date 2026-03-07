use std::io::Write;

use tempfile::NamedTempFile;
use viewer_core::archive::OoxmlArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use format_docx::{
    layout_document, layout_header_footers, parse_docx, parse_page_boxes,
    parse_paragraph_blocks, parse_section_layouts, parse_style_catalog,
};
use viewer_core::model::{Block, TableCellMerge};

#[test]
fn parses_main_optional_and_media_entry_points() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rStyle" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
              <Relationship Id="rNumbering" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>
              <Relationship Id="rHeader" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/>
              <Relationship Id="rFooter" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>
              <Relationship Id="rImage" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.png"/>
            </Relationships>"#,
        ),
        ("word/styles.xml", "<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
        ("word/numbering.xml", "<w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
        ("word/header1.xml", "<w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
        ("word/footer1.xml", "<w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
        ("word/media/image1.png", "fakepng"),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");

    let package = parse_docx(&archive).expect("docx package should parse");

    assert_eq!(package.main_document, "word/document.xml");
    assert_eq!(package.styles.as_deref(), Some("word/styles.xml"));
    assert_eq!(package.numbering.as_deref(), Some("word/numbering.xml"));
    assert_eq!(package.headers, vec!["word/header1.xml"]);
    assert_eq!(package.footers, vec!["word/footer1.xml"]);
    assert_eq!(package.media.len(), 1);
    assert_eq!(package.media[0].resolved_target, "word/media/image1.png");
  }

#[test]
fn missing_optional_parts_are_treated_as_empty() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");

    let package = parse_docx(&archive).expect("docx package should parse");

    assert_eq!(package.main_document, "word/document.xml");
    assert!(package.styles.is_none());
    assert!(package.numbering.is_none());
    assert!(package.headers.is_empty());
    assert!(package.footers.is_empty());
    assert!(package.media.is_empty());
}

#[test]
fn parses_styled_paragraph_runs() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p>
                  <w:r>
                    <w:rPr>
                      <w:rFonts w:ascii="Arial"/>
                      <w:sz w:val="28"/>
                      <w:b/>
                      <w:color w:val="FF0000"/>
                    </w:rPr>
                    <w:t>Styled</w:t>
                  </w:r>
                </w:p>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/styles.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rStyle" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
            </Relationships>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");

    let blocks = parse_paragraph_blocks(&archive, &package).expect("paragraphs should parse");

    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph { runs, list } => {
            assert_eq!(runs.len(), 1);
            assert_eq!(runs[0].text, "Styled");
            assert!(list.is_none());
            assert_eq!(runs[0].style.font_family, "Arial");
            assert_eq!(runs[0].style.font_size, 14.0);
            assert!(runs[0].style.bold);
            assert!(!runs[0].style.italic);
            assert_eq!(runs[0].style.color_hex, "#FF0000");
        }
        other => panic!("expected paragraph block, got {other:?}"),
    }
}

#[test]
fn parses_mixed_runs_and_line_breaks() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p>
                  <w:r><w:t>Hello</w:t></w:r>
                  <w:r>
                    <w:rPr><w:i/></w:rPr>
                    <w:t>World</w:t>
                    <w:br/>
                    <w:t>Again</w:t>
                  </w:r>
                </w:p>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/styles.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rStyle" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
            </Relationships>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");

    let blocks = parse_paragraph_blocks(&archive, &package).expect("paragraphs should parse");

    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph { runs, list } => {
            assert_eq!(runs.len(), 2);
            assert!(list.is_none());
            assert_eq!(runs[0].text, "Hello");
            assert_eq!(runs[0].style.font_family, "Times New Roman");
            assert!(!runs[0].style.bold);
            assert_eq!(runs[1].text, "World\nAgain");
            assert!(runs[1].style.italic);
            assert_eq!(runs[1].style.color_hex, "#000000");
        }
        other => panic!("expected paragraph block, got {other:?}"),
    }
}

#[test]
fn resolves_based_on_style_chain_from_styles_xml() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
              <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p>
                  <w:r>
                    <w:rPr><w:rStyle w:val="Emphasis"/></w:rPr>
                    <w:t>Styled chain</w:t>
                  </w:r>
                </w:p>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/styles.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:docDefaults>
                <w:rPrDefault>
                  <w:rPr>
                    <w:rFonts w:ascii="Calibri"/>
                    <w:sz w:val="22"/>
                    <w:color w:val="333333"/>
                  </w:rPr>
                </w:rPrDefault>
              </w:docDefaults>
              <w:style w:type="character" w:styleId="BaseChar">
                <w:rPr>
                  <w:i/>
                  <w:color w:val="00AA00"/>
                </w:rPr>
              </w:style>
              <w:style w:type="character" w:styleId="Emphasis">
                <w:basedOn w:val="BaseChar"/>
                <w:rPr>
                  <w:b/>
                  <w:rFonts w:ascii="Aptos"/>
                </w:rPr>
              </w:style>
            </w:styles>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rStyle" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
            </Relationships>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");

    let catalog = parse_style_catalog(&archive, &package).expect("styles should parse");
    assert_eq!(catalog.default_run_style.font_family.as_deref(), Some("Calibri"));

    let blocks = parse_paragraph_blocks(&archive, &package).expect("paragraphs should parse");

    match &blocks[0] {
        Block::Paragraph { runs, list } => {
            assert_eq!(runs[0].style.font_family, "Aptos");
            assert!(list.is_none());
            assert_eq!(runs[0].style.font_size, 11.0);
            assert!(runs[0].style.bold);
            assert!(runs[0].style.italic);
            assert_eq!(runs[0].style.color_hex, "#00AA00");
        }
        other => panic!("expected paragraph block, got {other:?}"),
    }
}

#[test]
fn direct_formatting_overrides_named_style_values() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
              <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p>
                  <w:r>
                    <w:rPr>
                      <w:rStyle w:val="Accent"/>
                      <w:color w:val="FF6600"/>
                      <w:sz w:val="30"/>
                    </w:rPr>
                    <w:t>Override me</w:t>
                  </w:r>
                </w:p>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/styles.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:docDefaults>
                <w:rPrDefault>
                  <w:rPr>
                    <w:rFonts w:ascii="Calibri"/>
                    <w:sz w:val="22"/>
                  </w:rPr>
                </w:rPrDefault>
              </w:docDefaults>
              <w:style w:type="character" w:styleId="Accent">
                <w:rPr>
                  <w:rFonts w:ascii="Aptos"/>
                  <w:color w:val="0000FF"/>
                </w:rPr>
              </w:style>
            </w:styles>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rStyle" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
            </Relationships>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");

    let blocks = parse_paragraph_blocks(&archive, &package).expect("paragraphs should parse");

    match &blocks[0] {
        Block::Paragraph { runs, list } => {
            assert_eq!(runs[0].style.font_family, "Aptos");
            assert!(list.is_none());
            assert_eq!(runs[0].style.font_size, 15.0);
            assert_eq!(runs[0].style.color_hex, "#FF6600");
        }
        other => panic!("expected paragraph block, got {other:?}"),
    }
}

#[test]
fn parses_nested_bullet_and_decimal_list_markers() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
              <Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p>
                  <w:pPr>
                    <w:numPr>
                      <w:ilvl w:val="0"/>
                      <w:numId w:val="7"/>
                    </w:numPr>
                  </w:pPr>
                  <w:r><w:t>Top bullet</w:t></w:r>
                </w:p>
                <w:p>
                  <w:pPr>
                    <w:numPr>
                      <w:ilvl w:val="1"/>
                      <w:numId w:val="7"/>
                    </w:numPr>
                  </w:pPr>
                  <w:r><w:t>Nested decimal</w:t></w:r>
                </w:p>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/numbering.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:abstractNum w:abstractNumId="1">
                <w:lvl w:ilvl="0"><w:numFmt w:val="bullet"/></w:lvl>
                <w:lvl w:ilvl="1"><w:numFmt w:val="decimal"/></w:lvl>
              </w:abstractNum>
              <w:num w:numId="7">
                <w:abstractNumId w:val="1"/>
              </w:num>
            </w:numbering>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rNumbering" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>
            </Relationships>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let blocks = parse_paragraph_blocks(&archive, &package).expect("paragraphs should parse");

    match &blocks[0] {
        Block::Paragraph { list, .. } => {
            let list = list.as_ref().expect("list marker");
            assert_eq!(list.kind, viewer_core::model::ListKind::Bullet);
            assert_eq!(list.level, 0);
            assert_eq!(list.num_id, 7);
        }
        other => panic!("expected paragraph block, got {other:?}"),
    }

    match &blocks[1] {
        Block::Paragraph { list, .. } => {
            let list = list.as_ref().expect("list marker");
            assert_eq!(list.kind, viewer_core::model::ListKind::Decimal);
            assert_eq!(list.level, 1);
        }
        other => panic!("expected paragraph block, got {other:?}"),
    }
}

#[test]
fn numbering_instance_override_uses_mapped_abstract_numbering() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
              <Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p>
                  <w:pPr>
                    <w:numPr>
                      <w:ilvl w:val="0"/>
                      <w:numId w:val="42"/>
                    </w:numPr>
                  </w:pPr>
                  <w:r><w:t>Overridden list</w:t></w:r>
                </w:p>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/numbering.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:abstractNum w:abstractNumId="3">
                <w:lvl w:ilvl="0"><w:numFmt w:val="decimal"/></w:lvl>
              </w:abstractNum>
              <w:abstractNum w:abstractNumId="9">
                <w:lvl w:ilvl="0"><w:numFmt w:val="bullet"/></w:lvl>
              </w:abstractNum>
              <w:num w:numId="42">
                <w:abstractNumId w:val="9"/>
              </w:num>
            </w:numbering>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rNumbering" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>
            </Relationships>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let blocks = parse_paragraph_blocks(&archive, &package).expect("paragraphs should parse");

    match &blocks[0] {
        Block::Paragraph { list, .. } => {
            let list = list.as_ref().expect("list marker");
            assert_eq!(list.kind, viewer_core::model::ListKind::Bullet);
            assert_eq!(list.num_id, 42);
        }
        other => panic!("expected paragraph block, got {other:?}"),
    }
}

#[test]
fn parses_basic_table_rows_cells_and_text() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:tbl>
                  <w:tr>
                    <w:tc>
                      <w:p><w:r><w:t>A1</w:t></w:r></w:p>
                    </w:tc>
                    <w:tc>
                      <w:p><w:r><w:t>B1</w:t></w:r></w:p>
                    </w:tc>
                  </w:tr>
                  <w:tr>
                    <w:tc>
                      <w:p><w:r><w:t>A2</w:t></w:r></w:p>
                    </w:tc>
                  </w:tr>
                </w:tbl>
              </w:body>
            </w:document>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let blocks = parse_paragraph_blocks(&archive, &package).expect("blocks should parse");

    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Table { rows } => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].cells.len(), 2);
            assert_eq!(rows[1].cells.len(), 1);
            match &rows[0].cells[0].blocks[0] {
                Block::Paragraph { runs, .. } => assert_eq!(runs[0].text, "A1"),
                other => panic!("expected paragraph block, got {other:?}"),
            }
        }
        other => panic!("expected table block, got {other:?}"),
    }
}

#[test]
fn parses_merged_cell_fallback_metadata() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:tbl>
                  <w:tr>
                    <w:tc>
                      <w:tcPr>
                        <w:gridSpan w:val="2"/>
                        <w:vMerge w:val="restart"/>
                      </w:tcPr>
                      <w:p><w:r><w:t>Merged start</w:t></w:r></w:p>
                    </w:tc>
                  </w:tr>
                  <w:tr>
                    <w:tc>
                      <w:tcPr><w:vMerge/></w:tcPr>
                      <w:p><w:r><w:t>Merged continue</w:t></w:r></w:p>
                    </w:tc>
                  </w:tr>
                </w:tbl>
              </w:body>
            </w:document>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let blocks = parse_paragraph_blocks(&archive, &package).expect("blocks should parse");

    match &blocks[0] {
        Block::Table { rows } => {
            assert_eq!(rows[0].cells[0].column_span, 2);
            assert_eq!(rows[0].cells[0].row_merge, Some(TableCellMerge::Restart));
            assert_eq!(rows[1].cells[0].row_merge, Some(TableCellMerge::Continue));
        }
        other => panic!("expected table block, got {other:?}"),
    }
}

#[test]
fn parses_inline_image_reference_from_drawing_relationship() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document
              xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
              xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
              xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"
              xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
              xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
              <w:body>
                <w:p>
                  <w:r>
                    <w:drawing>
                      <wp:inline>
                        <wp:docPr id="1" name="Diagram" descr="System diagram"/>
                        <a:graphic>
                          <a:graphicData>
                            <pic:pic>
                              <pic:blipFill>
                                <a:blip r:embed="rImage1"/>
                              </pic:blipFill>
                            </pic:pic>
                          </a:graphicData>
                        </a:graphic>
                      </wp:inline>
                    </w:drawing>
                  </w:r>
                </w:p>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rImage1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.png"/>
            </Relationships>"#,
        ),
        ("word/media/image1.png", "fakepng"),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let blocks = parse_paragraph_blocks(&archive, &package).expect("blocks should parse");

    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Image { image } => {
            assert_eq!(image.resource_id, "word/media/image1.png");
            assert_eq!(image.description.as_deref(), Some("System diagram"));
            assert_eq!(image.content_type.as_deref(), Some("image/png"));
        }
        other => panic!("expected image block, got {other:?}"),
    }
}

#[test]
fn missing_media_relationship_for_drawing_fails() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document
              xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
              xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
              xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"
              xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
              <w:body>
                <w:p>
                  <w:r>
                    <w:drawing>
                      <a:graphic>
                        <a:graphicData>
                          <pic:pic>
                            <pic:blipFill>
                              <a:blip r:embed="rMissing"/>
                            </pic:blipFill>
                          </pic:pic>
                        </a:graphicData>
                      </a:graphic>
                    </w:drawing>
                  </w:r>
                </w:p>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");

    let error = parse_paragraph_blocks(&archive, &package).expect_err("missing media should fail");

    assert!(matches!(error, viewer_core::ViewerError::InvalidDocument));
}

#[test]
fn parses_different_first_page_header_footer_references() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
              <w:body>
                <w:p><w:r><w:t>Page 1</w:t></w:r></w:p>
                <w:sectPr>
                  <w:headerReference w:type="first" r:id="rFirstHeader"/>
                  <w:headerReference w:type="default" r:id="rDefaultHeader"/>
                  <w:footerReference w:type="first" r:id="rFirstFooter"/>
                </w:sectPr>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rFirstHeader" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="first-header.xml"/>
              <Relationship Id="rDefaultHeader" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="default-header.xml"/>
              <Relationship Id="rFirstFooter" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="first-footer.xml"/>
            </Relationships>"#,
        ),
        ("word/first-header.xml", "<w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
        ("word/default-header.xml", "<w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
        ("word/first-footer.xml", "<w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let sections = parse_section_layouts(&archive, &package).expect("section layouts should parse");

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].headers.len(), 2);
    assert_eq!(sections[0].headers[0].target, "word/first-header.xml");
    assert_eq!(sections[0].headers[1].target, "word/default-header.xml");
    assert_eq!(sections[0].footers[0].target, "word/first-footer.xml");
}

#[test]
fn parses_section_break_header_footer_switch() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
              <w:body>
                <w:p>
                  <w:r><w:t>Section 1</w:t></w:r>
                  <w:pPr>
                    <w:sectPr>
                      <w:headerReference w:type="default" r:id="rHeader1"/>
                    </w:sectPr>
                  </w:pPr>
                </w:p>
                <w:p><w:r><w:t>Section 2</w:t></w:r></w:p>
                <w:sectPr>
                  <w:headerReference w:type="default" r:id="rHeader2"/>
                  <w:footerReference w:type="default" r:id="rFooter2"/>
                </w:sectPr>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rHeader1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="section1-header.xml"/>
              <Relationship Id="rHeader2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="section2-header.xml"/>
              <Relationship Id="rFooter2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="section2-footer.xml"/>
            </Relationships>"#,
        ),
        ("word/section1-header.xml", "<w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
        ("word/section2-header.xml", "<w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
        ("word/section2-footer.xml", "<w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let sections = parse_section_layouts(&archive, &package).expect("section layouts should parse");

    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].headers[0].target, "word/section1-header.xml");
    assert!(sections[0].footers.is_empty());
    assert_eq!(sections[1].headers[0].target, "word/section2-header.xml");
    assert_eq!(sections[1].footers[0].target, "word/section2-footer.xml");
}

#[test]
fn parses_page_metrics_from_section_properties() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:sectPr>
                  <w:pgSz w:w="12240" w:h="15840"/>
                  <w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800"/>
                </w:sectPr>
              </w:body>
            </w:document>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let pages = parse_page_boxes(&archive, &package).expect("page boxes should parse");

    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].width, 612.0);
    assert_eq!(pages[0].height, 792.0);
    assert_eq!(pages[0].margins.top, 72.0);
    assert_eq!(pages[0].margins.left, 90.0);
    assert_eq!(pages[0].content.x, 90.0);
    assert_eq!(pages[0].content.y, 72.0);
    assert_eq!(pages[0].content.width, 432.0);
    assert_eq!(pages[0].content.height, 648.0);
}

#[test]
fn lays_out_long_paragraph_across_multiple_pages() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p>
                  <w:r><w:t>alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau</w:t></w:r>
                </w:p>
                <w:sectPr>
                  <w:pgSz w:w="2400" w:h="1200"/>
                  <w:pgMar w:top="120" w:right="120" w:bottom="120" w:left="120"/>
                </w:sectPr>
              </w:body>
            </w:document>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let pages = layout_document(&archive, &package).expect("layout should succeed");

    assert!(pages.len() >= 2);
    assert_eq!(pages[0].page_index, 0);
    assert_eq!(pages[1].page_index, 1);
    assert!(!pages[0].blocks.is_empty());
    assert!(!pages[1].blocks.is_empty());
}

#[test]
fn explicit_page_break_starts_new_page() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p>
                  <w:r><w:t>First page</w:t></w:r>
                  <w:r><w:br w:type="page"/></w:r>
                  <w:r><w:t>Second page</w:t></w:r>
                </w:p>
                <w:sectPr>
                  <w:pgSz w:w="12240" w:h="15840"/>
                  <w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/>
                </w:sectPr>
              </w:body>
            </w:document>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let pages = layout_document(&archive, &package).expect("layout should succeed");

    assert_eq!(pages.len(), 2);
    match &pages[0].blocks[0] {
        format_docx::LaidOutBlock::Paragraph { lines, .. } => {
            assert!(lines.iter().any(|line| line.text.contains("First page")));
        }
        other => panic!("expected paragraph layout, got {other:?}"),
    }
    match &pages[1].blocks[0] {
        format_docx::LaidOutBlock::Paragraph { lines, .. } => {
            assert!(lines.iter().any(|line| line.text.contains("Second page")));
        }
        other => panic!("expected paragraph layout, got {other:?}"),
    }
}

#[test]
fn lays_out_header_footer_inside_margin_regions() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
              <w:body>
                <w:sectPr>
                  <w:headerReference w:type="default" r:id="rHeader"/>
                  <w:footerReference w:type="default" r:id="rFooter"/>
                  <w:pgSz w:w="12240" w:h="15840"/>
                  <w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/>
                </w:sectPr>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rHeader" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header.xml"/>
              <Relationship Id="rFooter" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer.xml"/>
            </Relationships>"#,
        ),
        ("word/header.xml", "<w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
        ("word/footer.xml", "<w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>"),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let layouts = layout_header_footers(&archive, &package).expect("header/footer layout should parse");

    assert_eq!(layouts.len(), 1);
    let layout = &layouts[0];
    assert_eq!(layout.headers[0].target, "word/header.xml");
    assert_eq!(layout.footers[0].target, "word/footer.xml");
    assert!(layout.headers[0].y < layout.page_box.content.y);
    assert!(layout.footers[0].y > layout.page_box.content.y + layout.page_box.content.height);
    assert_eq!(layout.headers[0].width, layout.page_box.content.width);
    assert_eq!(layout.footers[0].width, layout.page_box.content.width);
}

#[test]
fn lays_out_image_in_document_flow() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document
              xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
              xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
              xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"
              xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
              xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
              <w:body>
                <w:p><w:r><w:t>Intro</w:t></w:r></w:p>
                <w:p>
                  <w:r>
                    <w:drawing>
                      <wp:inline>
                        <wp:docPr id="1" name="Preview" descr="Flow image"/>
                        <a:graphic>
                          <a:graphicData>
                            <pic:pic>
                              <pic:blipFill>
                                <a:blip r:embed="rImage1"/>
                              </pic:blipFill>
                            </pic:pic>
                          </a:graphicData>
                        </a:graphic>
                      </wp:inline>
                    </w:drawing>
                  </w:r>
                </w:p>
                <w:p><w:r><w:t>Outro</w:t></w:r></w:p>
                <w:sectPr>
                  <w:pgSz w:w="2400" w:h="3000"/>
                  <w:pgMar w:top="120" w:right="120" w:bottom="120" w:left="120"/>
                </w:sectPr>
              </w:body>
            </w:document>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rImage1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.png"/>
            </Relationships>"#,
        ),
        ("word/media/image1.png", "fakepng"),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let pages = layout_document(&archive, &package).expect("layout should succeed");

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].blocks.len(), 2);
    match &pages[0].blocks[0] {
        format_docx::LaidOutBlock::Paragraph { lines, .. } => {
            assert!(lines[0].text.contains("Intro"));
        }
        other => panic!("expected paragraph layout, got {other:?}"),
    }
    let image_y = match &pages[0].blocks[1] {
        format_docx::LaidOutBlock::Image {
            resource_id,
            x,
            y,
            width,
            height,
        } => {
            assert_eq!(resource_id, "word/media/image1.png");
            assert_eq!(*x, pages[0].page_box.content.x);
            assert_eq!(*width, pages[0].page_box.content.width);
            assert_eq!(*height, 96.0);
            *y
        }
        other => panic!("expected image layout, got {other:?}"),
    };
    assert_eq!(pages[1].blocks.len(), 1);
    match &pages[1].blocks[0] {
        format_docx::LaidOutBlock::Paragraph { lines, .. } => {
            assert!(lines[0].text.contains("Outro"));
            assert!(pages[1].page_index > pages[0].page_index);
            assert!(image_y >= pages[0].page_box.content.y);
        }
        other => panic!("expected paragraph layout, got {other:?}"),
    }
}

#[test]
fn splits_table_rows_across_pages_when_needed() {
    let file = create_docx_fixture(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:tbl>
                  <w:tr><w:tc><w:p><w:r><w:t>R1</w:t></w:r></w:p></w:tc></w:tr>
                  <w:tr><w:tc><w:p><w:r><w:t>R2</w:t></w:r></w:p></w:tc></w:tr>
                  <w:tr><w:tc><w:p><w:r><w:t>R3</w:t></w:r></w:p></w:tc></w:tr>
                  <w:tr><w:tc><w:p><w:r><w:t>R4</w:t></w:r></w:p></w:tc></w:tr>
                </w:tbl>
                <w:sectPr>
                  <w:pgSz w:w="2400" w:h="1440"/>
                  <w:pgMar w:top="120" w:right="120" w:bottom="120" w:left="120"/>
                </w:sectPr>
              </w:body>
            </w:document>"#,
        ),
    ]);
    let archive = OoxmlArchive::open_path(file.path()).expect("docx archive should open");
    let package = parse_docx(&archive).expect("docx package should parse");
    let pages = layout_document(&archive, &package).expect("layout should succeed");

    assert_eq!(pages.len(), 2);
    match &pages[0].blocks[0] {
        format_docx::LaidOutBlock::Table { rows, y, height, .. } => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].cells.len(), 1);
            assert_eq!(rows[1].cells.len(), 1);
            assert_eq!(*y, pages[0].page_box.content.y);
            assert_eq!(*height, 48.0);
        }
        other => panic!("expected table layout, got {other:?}"),
    }
    match &pages[1].blocks[0] {
        format_docx::LaidOutBlock::Table { rows, y, height, .. } => {
            assert_eq!(rows.len(), 2);
            assert_eq!(*y, pages[1].page_box.content.y);
            assert_eq!(*height, 48.0);
            assert!(rows[0].y < rows[1].y);
        }
        other => panic!("expected table layout, got {other:?}"),
    }
}

fn create_docx_fixture(entries: &[(&str, &str)]) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("temp zip");
    {
        let writer = file.as_file_mut();
        let mut zip = ZipWriter::new(writer);

        for (name, contents) in entries {
            zip.start_file(*name, SimpleFileOptions::default())
                .expect("zip entry should start");
            zip.write_all(contents.as_bytes())
                .expect("zip entry should write");
        }

        zip.finish().expect("zip should finish");
    }

    file
}

use std::fs;
use std::io::Write;

use format_xlsx::{
    parse_cell_style_subset, parse_xlsx, XlsxHorizontalAlignment, XlsxVerticalAlignment,
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
fn parses_cell_style_subset_from_styles_part() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("styles.xlsx");
    create_zip(
        &path,
        &[
            (
                "_rels/.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>
"#,
            ),
            (
                "xl/workbook.xml",
                br#"
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
          xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="Sheet1" sheetId="1" r:id="rIdSheet1"/>
  </sheets>
</workbook>
"#,
            ),
            (
                "xl/_rels/workbook.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdSheet1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rIdStyles" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>
"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                br#"
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData/>
</worksheet>
"#,
            ),
            (
                "xl/styles.xml",
                br#"
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <numFmts count="1">
    <numFmt numFmtId="164" formatCode="yyyy-mm-dd"/>
  </numFmts>
  <fonts count="2">
    <font>
      <sz val="11"/>
      <name val="Aptos"/>
      <color rgb="FF112233"/>
    </font>
    <font>
      <b/>
      <i/>
      <u/>
      <sz val="14"/>
      <name val="Pretendard"/>
      <color rgb="CC445566"/>
    </font>
  </fonts>
  <fills count="2">
    <fill><patternFill patternType="none"/></fill>
    <fill>
      <patternFill patternType="solid">
        <fgColor rgb="FFABCDEF"/>
        <bgColor rgb="FF010203"/>
      </patternFill>
    </fill>
  </fills>
  <cellXfs count="2">
    <xf numFmtId="0" fontId="0" fillId="0"/>
    <xf numFmtId="164" fontId="1" fillId="1" applyNumberFormat="1" applyAlignment="1">
      <alignment horizontal="center" vertical="center" wrapText="1"/>
    </xf>
  </cellXfs>
</styleSheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let styles = parse_cell_style_subset(&archive, &workbook).expect("styles should parse");

    assert_eq!(workbook.styles_part.as_deref(), Some("xl/styles.xml"));
    assert_eq!(styles.part_name.as_deref(), Some("xl/styles.xml"));
    assert_eq!(styles.number_formats.len(), 1);
    assert_eq!(styles.number_formats[0].id, 164);
    assert_eq!(styles.number_formats[0].code, "yyyy-mm-dd");
    assert_eq!(styles.fonts.len(), 2);
    assert_eq!(styles.fonts[0].font_name.as_deref(), Some("Aptos"));
    assert_eq!(styles.fonts[0].font_size_points, Some(11.0));
    assert_eq!(styles.fonts[0].color_hex.as_deref(), Some("#112233"));
    assert!(styles.fonts[1].bold);
    assert!(styles.fonts[1].italic);
    assert!(styles.fonts[1].underline);
    assert_eq!(styles.fonts[1].color_hex.as_deref(), Some("#445566CC"));
    assert_eq!(styles.fills.len(), 2);
    assert_eq!(styles.fills[1].pattern_type.as_deref(), Some("solid"));
    assert_eq!(
        styles.fills[1].foreground_color_hex.as_deref(),
        Some("#ABCDEF")
    );
    assert_eq!(
        styles.fills[1].background_color_hex.as_deref(),
        Some("#010203")
    );
    assert_eq!(styles.cell_formats.len(), 2);
    assert_eq!(styles.cell_formats[1].num_fmt_id, 164);
    assert_eq!(
        styles.cell_formats[1].number_format_code.as_deref(),
        Some("yyyy-mm-dd")
    );
    assert_eq!(styles.cell_formats[1].font_id, 1);
    assert_eq!(styles.cell_formats[1].fill_id, 1);
    assert!(styles.cell_formats[1].apply_number_format);
    assert!(styles.cell_formats[1].apply_alignment);
    assert_eq!(
        styles.cell_formats[1].horizontal_alignment,
        Some(XlsxHorizontalAlignment::Center)
    );
    assert_eq!(
        styles.cell_formats[1].vertical_alignment,
        Some(XlsxVerticalAlignment::Center)
    );
    assert!(styles.cell_formats[1].wrap_text);
}

#[test]
fn missing_styles_relationship_returns_empty_catalog() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("no-styles.xlsx");
    create_zip(
        &path,
        &[
            (
                "_rels/.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>
"#,
            ),
            (
                "xl/workbook.xml",
                br#"
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
          xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="Sheet1" sheetId="1" r:id="rIdSheet1"/>
  </sheets>
</workbook>
"#,
            ),
            (
                "xl/_rels/workbook.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdSheet1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>
"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                br#"
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData/>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let styles = parse_cell_style_subset(&archive, &workbook).expect("styles should parse");

    assert!(styles.part_name.is_none());
    assert!(styles.number_formats.is_empty());
    assert!(styles.fonts.is_empty());
    assert!(styles.fills.is_empty());
    assert!(styles.cell_formats.is_empty());
}

#[test]
fn invalid_alignment_value_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("bad-styles.xlsx");
    create_zip(
        &path,
        &[
            (
                "_rels/.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>
"#,
            ),
            (
                "xl/workbook.xml",
                br#"
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
          xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="Sheet1" sheetId="1" r:id="rIdSheet1"/>
  </sheets>
</workbook>
"#,
            ),
            (
                "xl/_rels/workbook.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdSheet1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rIdStyles" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>
"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                br#"
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData/>
</worksheet>
"#,
            ),
            (
                "xl/styles.xml",
                br#"
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <cellXfs count="1">
    <xf numFmtId="0" fontId="0" fillId="0" applyAlignment="1">
      <alignment horizontal="diagonal"/>
    </xf>
  </cellXfs>
</styleSheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let error = parse_cell_style_subset(&archive, &workbook).expect_err("invalid alignment");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

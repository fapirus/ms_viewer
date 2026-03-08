use std::fs;
use std::io::Write;

use format_xlsx::parse_shared_strings;
use format_xlsx::parse_worksheet_cells;
use format_xlsx::parse_xlsx;
use format_xlsx::XlsxCellValue;
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
fn parses_formula_cells_with_cached_values() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("formula-cached-values.xlsx");
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
  <Relationship Id="rIdSharedStrings" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
</Relationships>
"#,
            ),
            (
                "xl/sharedStrings.xml",
                br#"
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <si><t>Summary</t></si>
</sst>
"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                br#"
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" t="s"><v>0</v></c>
      <c r="B1" s="2"><f>SUM(B2:B4)</f><v>42</v></c>
      <c r="C1" t="str"><f>CONCAT(A1," total")</f><v>Summary total</v></c>
      <c r="D1" t="b"><v>1</v></c>
    </row>
  </sheetData>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let shared_strings = parse_shared_strings(&archive, &workbook).expect("shared strings");
    let worksheet_cells =
        parse_worksheet_cells(&archive, &workbook, &shared_strings).expect("worksheet cells");

    assert_eq!(worksheet_cells.len(), 1);
    assert_eq!(worksheet_cells[0].part_name, "xl/worksheets/sheet1.xml");
    assert_eq!(worksheet_cells[0].cells.len(), 4);

    assert_eq!(
        worksheet_cells[0].cells[0].value,
        Some(XlsxCellValue::Text("Summary".to_string()))
    );
    assert_eq!(
        worksheet_cells[0].cells[1].formula.as_deref(),
        Some("SUM(B2:B4)")
    );
    assert_eq!(
        worksheet_cells[0].cells[1].value,
        Some(XlsxCellValue::Number("42".to_string()))
    );
    assert_eq!(worksheet_cells[0].cells[1].style_index, Some(2));
    assert_eq!(
        worksheet_cells[0].cells[2].value,
        Some(XlsxCellValue::Text("Summary total".to_string()))
    );
    assert_eq!(
        worksheet_cells[0].cells[3].value,
        Some(XlsxCellValue::Boolean(true))
    );
}

#[test]
fn formula_without_cached_value_keeps_formula_only() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("formula-without-cache.xlsx");
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
  <sheetData>
    <row r="1">
      <c r="A1"><f>NOW()</f></c>
    </row>
  </sheetData>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let worksheet_cells =
        parse_worksheet_cells(&archive, &workbook, &[]).expect("worksheet cells");

    assert_eq!(worksheet_cells[0].cells.len(), 1);
    assert_eq!(worksheet_cells[0].cells[0].formula.as_deref(), Some("NOW()"));
    assert_eq!(worksheet_cells[0].cells[0].value, None);
}

#[test]
fn invalid_shared_string_index_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("bad-shared-string-index.xlsx");
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
  <Relationship Id="rIdSharedStrings" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
</Relationships>
"#,
            ),
            (
                "xl/sharedStrings.xml",
                br#"
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <si><t>Only one</t></si>
</sst>
"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                br#"
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" t="s"><v>3</v></c>
    </row>
  </sheetData>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let shared_strings = parse_shared_strings(&archive, &workbook).expect("shared strings");
    let error = parse_worksheet_cells(&archive, &workbook, &shared_strings)
        .expect_err("shared string index should be invalid");

    assert!(matches!(error, ViewerError::InvalidDocument));
}


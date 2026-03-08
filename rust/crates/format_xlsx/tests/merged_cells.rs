use std::fs;
use std::io::Write;

use format_xlsx::parse_merged_cells;
use format_xlsx::parse_xlsx;
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
fn parses_worksheet_merged_cell_ranges() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("merged-cells.xlsx");
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
  <mergeCells count="2">
    <mergeCell ref="A1:C3"/>
    <mergeCell ref="D5:D8"/>
  </mergeCells>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let merged_cells = parse_merged_cells(&archive, &workbook).expect("merged cells");

    assert_eq!(merged_cells.len(), 1);
    assert_eq!(merged_cells[0].part_name, "xl/worksheets/sheet1.xml");
    assert_eq!(merged_cells[0].ranges.len(), 2);
    assert_eq!(merged_cells[0].ranges[0].reference, "A1:C3");
    assert_eq!(merged_cells[0].ranges[0].start_row, 1);
    assert_eq!(merged_cells[0].ranges[0].start_column, 1);
    assert_eq!(merged_cells[0].ranges[0].end_row, 3);
    assert_eq!(merged_cells[0].ranges[0].end_column, 3);
    assert_eq!(merged_cells[0].ranges[1].reference, "D5:D8");
    assert_eq!(merged_cells[0].ranges[1].start_column, 4);
    assert_eq!(merged_cells[0].ranges[1].end_row, 8);
}

#[test]
fn worksheet_without_merge_cells_returns_empty_ranges() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("no-merged-cells.xlsx");
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
    let merged_cells = parse_merged_cells(&archive, &workbook).expect("merged cells");

    assert!(merged_cells[0].ranges.is_empty());
}

#[test]
fn invalid_merged_cell_reference_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("bad-merged-cells.xlsx");
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
  <mergeCells count="1">
    <mergeCell ref="C3:A1"/>
  </mergeCells>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let error = parse_merged_cells(&archive, &workbook).expect_err("invalid merged range");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

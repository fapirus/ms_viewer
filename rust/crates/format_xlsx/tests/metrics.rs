use std::fs;
use std::io::Write;

use format_xlsx::{parse_row_column_metrics, parse_xlsx};
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
fn parses_default_and_custom_row_column_metrics() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sheet-metrics.xlsx");
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
  <sheetFormatPr defaultRowHeight="15" defaultColWidth="8.43"/>
  <cols>
    <col min="1" max="1" width="12.5" customWidth="1"/>
    <col min="2" max="4" width="20" hidden="1" customWidth="1"/>
  </cols>
  <sheetData>
    <row r="1" ht="24" customHeight="1"/>
    <row r="2" hidden="1"/>
  </sheetData>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let metrics = parse_row_column_metrics(&archive, &workbook).expect("metrics should parse");

    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].part_name, "xl/worksheets/sheet1.xml");
    assert_eq!(metrics[0].default_row_height_points, Some(15.0));
    assert_eq!(metrics[0].default_column_width, Some(8.43));
    assert_eq!(metrics[0].columns.len(), 2);
    assert_eq!(metrics[0].columns[0].min, 1);
    assert_eq!(metrics[0].columns[0].max, 1);
    assert_eq!(metrics[0].columns[0].width, Some(12.5));
    assert!(metrics[0].columns[0].custom_width);
    assert!(metrics[0].columns[1].hidden);
    assert_eq!(metrics[0].rows.len(), 2);
    assert_eq!(metrics[0].rows[0].index, 1);
    assert_eq!(metrics[0].rows[0].height_points, Some(24.0));
    assert!(metrics[0].rows[0].custom_height);
    assert_eq!(metrics[0].rows[1].index, 2);
    assert!(metrics[0].rows[1].hidden);
}

#[test]
fn malformed_decimal_metric_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("bad-metrics.xlsx");
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
  <sheetFormatPr defaultRowHeight="oops"/>
  <sheetData/>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let error = parse_row_column_metrics(&archive, &workbook).expect_err("invalid decimal");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

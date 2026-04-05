use std::fs;
use std::io::Write;

use format_xlsx::{parse_shared_strings, parse_xlsx};
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
fn parses_simple_and_rich_shared_strings() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("shared-strings.xlsx");
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
  <Relationship Id="rIdShared" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
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
                "xl/sharedStrings.xml",
                br#"
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="2" uniqueCount="2">
  <si><t>Quarterly Summary</t></si>
  <si>
    <r><t>Need </t></r>
    <r><t>web </t></r>
    <r><t>access</t></r>
  </si>
</sst>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let shared_strings = parse_shared_strings(&archive, &workbook).expect("shared strings");

    assert_eq!(
        workbook.shared_strings_part.as_deref(),
        Some("xl/sharedStrings.xml")
    );
    assert_eq!(
        shared_strings,
        vec![
            "Quarterly Summary".to_string(),
            "Need web access".to_string()
        ]
    );
}

#[test]
fn missing_shared_strings_relationship_returns_empty_table() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("no-shared-strings.xlsx");
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
    let shared_strings = parse_shared_strings(&archive, &workbook).expect("shared strings");

    assert!(shared_strings.is_empty());
}

#[test]
fn missing_shared_strings_part_fails() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("missing-shared-strings-part.xlsx");
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
  <Relationship Id="rIdShared" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
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
    let error = parse_xlsx(&archive).expect_err("missing shared strings part should fail");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

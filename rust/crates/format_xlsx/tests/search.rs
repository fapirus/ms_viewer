use std::fs;
use std::io::Write;

use format_xlsx::build_search_pages;
use format_xlsx::parse_cell_style_subset;
use format_xlsx::parse_shared_strings;
use format_xlsx::parse_xlsx;
use format_xlsx::search_workbook;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;

use viewer_core::archive::OoxmlArchive;

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
fn builds_sheet_search_pages_from_cell_values() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sheet-search-pages.xlsx");
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
    <sheet name="Sheet2" sheetId="2" r:id="rIdSheet2"/>
  </sheets>
</workbook>
"#,
            ),
            (
                "xl/_rels/workbook.xml.rels",
                br#"
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdSheet1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rIdSheet2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/>
  <Relationship Id="rIdSharedStrings" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
</Relationships>
"#,
            ),
            (
                "xl/sharedStrings.xml",
                br#"
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <si><t>Revenue</t></si>
  <si><t>North Region</t></si>
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
      <c r="B1" t="s"><v>1</v></c>
    </row>
    <row r="2">
      <c r="A2"><v>120</v></c>
      <c r="B2"><f>SUM(B3:B4)</f><v>340</v></c>
    </row>
  </sheetData>
</worksheet>
"#,
            ),
            (
                "xl/worksheets/sheet2.xml",
                br#"
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" t="inlineStr"><is><t>Status</t></is></c>
      <c r="B1" t="b"><v>1</v></c>
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
    let styles = parse_cell_style_subset(&archive, &workbook).expect("styles");
    let pages =
        build_search_pages(&archive, &workbook, &shared_strings, &styles).expect("search pages");

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].page_index, 0);
    assert_eq!(pages[0].text, "Revenue\tNorth Region\n120\t340");
    assert_eq!(pages[1].text, "Status\tTRUE");
}

#[test]
fn workbook_search_is_case_insensitive() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sheet-search.xlsx");
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
      <c r="A1" t="inlineStr"><is><t>Quarterly Revenue</t></is></c>
    </row>
  </sheetData>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let styles = parse_cell_style_subset(&archive, &workbook).expect("styles");
    let matches = search_workbook(&archive, &workbook, &[], &styles, "revenue").expect("search");

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].page_index, 0);
    assert_eq!(matches[0].query, "revenue");
    assert!(matches[0].preview.contains("Quarterly Revenue"));
}

#[test]
fn search_pages_use_formatted_number_and_date_values() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sheet-search-formatted.xlsx");
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
  <workbookPr date1904="1"/>
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
                "xl/styles.xml",
                br##"
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <numFmts count="2">
    <numFmt numFmtId="164" formatCode="#,##0.00"/>
    <numFmt numFmtId="165" formatCode="yyyy-mm-dd"/>
  </numFmts>
  <fonts count="1"><font><name val="Calibri"/><sz val="11"/></font></fonts>
  <fills count="2">
    <fill><patternFill patternType="none"/></fill>
    <fill><patternFill patternType="gray125"/></fill>
  </fills>
  <cellXfs count="3">
    <xf numFmtId="164" fontId="0" fillId="0" applyNumberFormat="1"/>
    <xf numFmtId="165" fontId="0" fillId="0" applyNumberFormat="1"/>
    <xf numFmtId="10" fontId="0" fillId="0" applyNumberFormat="1"/>
  </cellXfs>
</styleSheet>
"##,
            ),
            (
                "xl/worksheets/sheet1.xml",
                br#"
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" s="0"><v>1234.5</v></c>
      <c r="B1" s="1"><v>1</v></c>
      <c r="C1" s="2"><v>0.125</v></c>
    </row>
  </sheetData>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let styles = parse_cell_style_subset(&archive, &workbook).expect("styles");
    let pages = build_search_pages(&archive, &workbook, &[], &styles).expect("search pages");

    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].text, "1,234.50\t1904-01-02\t12.50%");
}

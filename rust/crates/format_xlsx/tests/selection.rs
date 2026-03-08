use std::fs;
use std::io::Write;

use format_xlsx::build_selection_sheet_models;
use format_xlsx::parse_cell_style_subset;
use format_xlsx::parse_shared_strings;
use format_xlsx::parse_xlsx;
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
fn selection_models_expose_text_only_anchors() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sheet-selection.xlsx");
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
  <Relationship Id="rIdStyles" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>
"#,
            ),
            (
                "xl/sharedStrings.xml",
                br#"
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <si><t>Alpha</t></si>
</sst>
"#,
            ),
            (
                "xl/styles.xml",
                br#"
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="1"><font><name val="Calibri"/><sz val="11"/></font></fonts>
  <fills count="2">
    <fill><patternFill patternType="none"/></fill>
    <fill><patternFill patternType="gray125"/></fill>
  </fills>
  <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0"/></cellXfs>
</styleSheet>
"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                br#"
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1:B2"/>
  <sheetFormatPr defaultRowHeight="15" defaultColWidth="8.43"/>
  <sheetData>
    <row r="1">
      <c r="A1" t="s"><v>0</v></c>
      <c r="B1"><v>42</v></c>
    </row>
    <row r="2">
      <c r="A2" t="inlineStr"><is><t>Beta</t></is></c>
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
    let models = build_selection_sheet_models(&archive, &workbook, &shared_strings, &styles)
        .expect("selection models");

    assert_eq!(models.len(), 1);
    let model = &models[0];
    let text_node_indexes: Vec<u32> = model
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| match node {
            RenderNode::Text(_) => Some(index as u32),
            _ => None,
        })
        .collect();
    let total_chars: usize = model
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(node) => Some(node.text.chars().count() + 1),
            _ => None,
        })
        .sum();

    assert_eq!(text_node_indexes.len(), 3);
    assert_eq!(model.selection_anchors.len(), total_chars);
    assert!(model
        .selection_anchors
        .iter()
        .all(|anchor| text_node_indexes.contains(&anchor.node_index)));
}


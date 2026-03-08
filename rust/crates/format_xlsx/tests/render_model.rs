use std::fs;
use std::io::Write;

use format_xlsx::build_sheet_render_model;
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
fn builds_sheet_render_model_from_cells_metrics_and_merges() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sheet-render.xlsx");
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
  <si><t>Quarterly Summary</t></si>
</sst>
"#,
            ),
            (
                "xl/styles.xml",
                br#"
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="2">
    <font>
      <name val="Calibri"/>
      <sz val="11"/>
      <color rgb="FF000000"/>
    </font>
    <font>
      <name val="Calibri"/>
      <sz val="12"/>
      <b/>
      <color rgb="FFFFFFFF"/>
    </font>
  </fonts>
  <fills count="3">
    <fill><patternFill patternType="none"/></fill>
    <fill><patternFill patternType="gray125"/></fill>
    <fill>
      <patternFill patternType="solid">
        <fgColor rgb="FF1F4E78"/>
        <bgColor rgb="FF1F4E78"/>
      </patternFill>
    </fill>
  </fills>
  <cellXfs count="2">
    <xf numFmtId="0" fontId="0" fillId="0"/>
    <xf numFmtId="0" fontId="1" fillId="2" applyAlignment="1">
      <alignment horizontal="center" vertical="center"/>
    </xf>
  </cellXfs>
</styleSheet>
"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                br#"
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1:B3"/>
  <sheetFormatPr defaultRowHeight="15" defaultColWidth="8.43"/>
  <cols>
    <col min="1" max="2" width="12" customWidth="1"/>
  </cols>
  <sheetData>
    <row r="1" ht="24" customHeight="1">
      <c r="A1" s="1" t="s"><v>0</v></c>
      <c r="B1" s="1"/>
    </row>
    <row r="2">
      <c r="A2"><f>SUM(A3:B3)</f><v>42</v></c>
      <c r="B2" t="inlineStr"><is><t>inline</t></is></c>
    </row>
    <row r="3">
      <c r="A3" t="b"><v>1</v></c>
      <c r="B3" t="e"><v>#DIV/0!</v></c>
    </row>
  </sheetData>
  <mergeCells count="1">
    <mergeCell ref="A1:B1"/>
  </mergeCells>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let shared_strings = parse_shared_strings(&archive, &workbook).expect("shared strings");
    let styles = parse_cell_style_subset(&archive, &workbook).expect("styles");
    let render_model =
        build_sheet_render_model(&archive, &workbook, &shared_strings, &styles, 0).expect("render model");

    assert_eq!(render_model.page_index, 0);
    assert!(render_model.width > 0.0);
    assert!(render_model.height > 0.0);

    let text_nodes: Vec<_> = render_model
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(text) => Some(text),
            _ => None,
        })
        .collect();
    let box_nodes: Vec<_> = render_model
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Box(node) => Some(node),
            _ => None,
        })
        .collect();

    assert!(text_nodes.iter().any(|node| node.text == "Quarterly Summary"));
    assert!(text_nodes.iter().any(|node| node.text == "42"));
    assert!(text_nodes.iter().any(|node| node.text == "inline"));
    assert!(text_nodes.iter().any(|node| node.text == "TRUE"));
    assert!(text_nodes.iter().any(|node| node.text == "#DIV/0!"));
    assert_eq!(box_nodes.len(), 5);

    let merged_header_box = box_nodes
        .iter()
        .find(|node| node.fill_color_hex.as_deref() == Some("#1F4E78"))
        .expect("merged header box");
    assert!(merged_header_box.bounds.width > 120.0);

    let header_text = text_nodes
        .iter()
        .find(|node| node.text == "Quarterly Summary")
        .expect("header text");
    assert_eq!(header_text.style.color_hex, "#FFFFFF");
    assert!(header_text.style.bold);
    assert!(!render_model.selection_anchors.is_empty());
}

use std::fs;
use std::io::Write;

use format_xlsx::build_sheet_render_model;
use format_xlsx::build_visible_window_render_model;
use format_xlsx::parse_cell_style_subset;
use format_xlsx::parse_shared_strings;
use format_xlsx::parse_xlsx;
use format_xlsx::XlsxVisibleWindow;
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

#[test]
fn visible_window_render_model_limits_rows_and_columns() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sheet-window.xlsx");
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
  <si><t>Header</t></si>
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
  <dimension ref="A1:C4"/>
  <sheetFormatPr defaultRowHeight="15" defaultColWidth="8.43"/>
  <sheetData>
    <row r="1">
      <c r="A1" t="s"><v>0</v></c>
      <c r="B1"><v>10</v></c>
      <c r="C1"><v>11</v></c>
    </row>
    <row r="2">
      <c r="A2"><v>20</v></c>
      <c r="B2"><v>21</v></c>
      <c r="C2"><v>22</v></c>
    </row>
    <row r="3">
      <c r="A3"><v>30</v></c>
      <c r="B3"><v>31</v></c>
      <c r="C3"><v>32</v></c>
    </row>
    <row r="4">
      <c r="A4"><v>40</v></c>
      <c r="B4"><v>41</v></c>
      <c r="C4"><v>42</v></c>
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
    let render_model = build_visible_window_render_model(
        &archive,
        &workbook,
        &shared_strings,
        &styles,
        0,
        &XlsxVisibleWindow {
            start_row: 2,
            start_column: 2,
            row_count: 2,
            column_count: 2,
        },
    )
    .expect("window render model");

    let text_values: Vec<_> = render_model
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(node) => Some(node.text.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(text_values, vec!["21", "22", "31", "32"]);
    assert!(render_model.width > 0.0);
    assert!(render_model.height > 0.0);
    assert!(render_model.selection_anchors.len() >= 8);
}

#[test]
fn hidden_rows_and_columns_do_not_consume_sheet_space() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sheet-hidden-metrics.xlsx");
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
  <dimension ref="A1:B2"/>
  <sheetFormatPr defaultRowHeight="15" defaultColWidth="8.43"/>
  <cols>
    <col min="1" max="1" width="12" customWidth="1"/>
    <col min="2" max="2" width="20" hidden="1" customWidth="1"/>
  </cols>
  <sheetData>
    <row r="1" ht="12" customHeight="1">
      <c r="A1" t="inlineStr"><is><t>visible</t></is></c>
      <c r="B1" t="inlineStr"><is><t>hidden column</t></is></c>
    </row>
    <row r="2" hidden="1">
      <c r="A2" t="inlineStr"><is><t>hidden row</t></is></c>
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
    let render_model =
        build_sheet_render_model(&archive, &workbook, &[], &styles, 0).expect("render model");

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

    assert_eq!(box_nodes.len(), 1);
    assert_eq!(text_nodes.len(), 1);
    assert_eq!(text_nodes[0].text, "visible");
    assert!((render_model.width - excel_column_width_to_pixels(12.0)).abs() < 0.1);
    assert!((render_model.height - (12.0 * (96.0 / 72.0))).abs() < 0.1);
}

#[test]
fn merged_cells_use_spanned_column_widths_and_row_heights() {
    let dir = tempdir().expect("tempdir should exist");
    let path = dir.path().join("sheet-merged-span.xlsx");
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
  <dimension ref="A1:B2"/>
  <sheetFormatPr defaultRowHeight="15" defaultColWidth="8.43"/>
  <cols>
    <col min="1" max="1" width="10" customWidth="1"/>
    <col min="2" max="2" width="15" customWidth="1"/>
  </cols>
  <sheetData>
    <row r="1" ht="18" customHeight="1">
      <c r="A1" t="inlineStr"><is><t>Merged title</t></is></c>
      <c r="B1"/>
    </row>
    <row r="2" ht="21" customHeight="1">
      <c r="A2"/>
      <c r="B2"/>
    </row>
  </sheetData>
  <mergeCells count="1">
    <mergeCell ref="A1:B2"/>
  </mergeCells>
</worksheet>
"#,
            ),
        ],
    );

    let archive = OoxmlArchive::open_path(&path).expect("archive should open");
    let workbook = parse_xlsx(&archive).expect("xlsx should parse");
    let styles = parse_cell_style_subset(&archive, &workbook).expect("styles");
    let render_model =
        build_sheet_render_model(&archive, &workbook, &[], &styles, 0).expect("render model");

    let merged_box = render_model
        .nodes
        .iter()
        .find_map(|node| match node {
            RenderNode::Box(node) => Some(node),
            _ => None,
        })
        .expect("merged cell box should exist");
    let merged_text = render_model
        .nodes
        .iter()
        .find_map(|node| match node {
            RenderNode::Text(node) => Some(node),
            _ => None,
        })
        .expect("merged cell text should exist");

    let expected_width = excel_column_width_to_pixels(10.0) + excel_column_width_to_pixels(15.0);
    let expected_height = (18.0 * (96.0 / 72.0)) + (21.0 * (96.0 / 72.0));

    assert!((merged_box.bounds.width - expected_width).abs() < 0.1);
    assert!((merged_box.bounds.height - expected_height).abs() < 0.1);
    assert!(merged_text.bounds.x >= merged_box.bounds.x);
    assert!(merged_text.bounds.y >= merged_box.bounds.y);
    assert!(merged_text.bounds.x + merged_text.bounds.width <= merged_box.bounds.x + merged_box.bounds.width);
    assert!(
        merged_text.bounds.y + merged_text.bounds.height
            <= merged_box.bounds.y + merged_box.bounds.height
    );
}

fn excel_column_width_to_pixels(width_units: f32) -> f32 {
    let max_digit_width = 7.0;
    let padding = 5.0;
    let truncation = (128.0_f32 / max_digit_width).floor();
    ((((256.0 * width_units) + truncation) / 256.0).floor() * max_digit_width + padding).max(0.0)
}

use std::path::PathBuf;

use format_xlsx::{
    build_search_pages, build_selection_sheet_models, build_sheet_render_model,
    build_visible_window_render_model, parse_cell_style_subset, parse_frozen_panes,
    parse_merged_cells, parse_shared_strings, parse_worksheet_cells, parse_xlsx, search_workbook,
    XlsxVisibleWindow,
};
use viewer_core::archive::OoxmlArchive;
use viewer_core::model::RenderNode;

fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(relative)
}

#[test]
fn review_fixture_basic_grid_opens_and_supports_render_and_search() {
    let archive = OoxmlArchive::open_path(fixture_path("fixtures/xlsx/xlsx_basic_grid.xlsx"))
        .expect("fixture archive");
    let workbook = parse_xlsx(&archive).expect("workbook");
    let shared_strings = parse_shared_strings(&archive, &workbook).expect("shared strings");
    let styles = parse_cell_style_subset(&archive, &workbook).expect("styles");

    assert_eq!(workbook.sheets.len(), 2);
    assert_eq!(workbook.sheets[0].name, "Summary");
    assert_eq!(workbook.sheets[1].name, "Detail");

    let page = build_sheet_render_model(&archive, &workbook, &shared_strings, &styles, 0)
        .expect("sheet render model");
    let text_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(node) => Some(node),
            _ => None,
        })
        .collect();
    assert!(text_nodes.iter().any(|node| node.text == "Revenue"));
    assert!(text_nodes.iter().any(|node| node.text == "North Region"));
    assert!(text_nodes.iter().any(|node| node.text == "340"));

    let pages = build_search_pages(&archive, &workbook, &shared_strings).expect("search pages");
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].text, "Revenue\tNorth Region\n120\t340");
    let matches = search_workbook(&archive, &workbook, &shared_strings, "status").expect("search");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].page_index, 1);
}

#[test]
fn review_fixture_merges_frozen_formulas_support_selection_and_window_render() {
    let archive = OoxmlArchive::open_path(fixture_path(
        "fixtures/xlsx/xlsx_merges_frozen_formulas.xlsx",
    ))
    .expect("fixture archive");
    let workbook = parse_xlsx(&archive).expect("workbook");
    let shared_strings = parse_shared_strings(&archive, &workbook).expect("shared strings");
    let styles = parse_cell_style_subset(&archive, &workbook).expect("styles");

    let worksheet_cells =
        parse_worksheet_cells(&archive, &workbook, &shared_strings).expect("worksheet cells");
    assert_eq!(worksheet_cells[0].cells.len(), 8);
    assert!(worksheet_cells[0]
        .cells
        .iter()
        .any(|cell| cell.formula.as_deref() == Some("SUM(A3:B4)") && cell.value.is_some()));

    let merged_cells = parse_merged_cells(&archive, &workbook).expect("merged cells");
    assert_eq!(merged_cells[0].ranges.len(), 1);
    assert_eq!(merged_cells[0].ranges[0].reference, "A1:B1");

    let frozen_panes = parse_frozen_panes(&archive, &workbook).expect("frozen panes");
    let pane = frozen_panes[0].pane.as_ref().expect("frozen pane");
    assert_eq!(pane.top_left_cell.as_deref(), Some("B2"));

    let page = build_sheet_render_model(&archive, &workbook, &shared_strings, &styles, 0)
        .expect("sheet render model");
    let selection_models =
        build_selection_sheet_models(&archive, &workbook, &shared_strings, &styles)
            .expect("selection models");
    assert_eq!(selection_models.len(), 1);
    assert!(!selection_models[0].selection_anchors.is_empty());
    let image_or_box_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Box(node) => Some(node),
            _ => None,
        })
        .collect();
    assert!(!image_or_box_nodes.is_empty());

    let window_page = build_visible_window_render_model(
        &archive,
        &workbook,
        &shared_strings,
        &styles,
        0,
        &XlsxVisibleWindow {
            start_row: 2,
            start_column: 1,
            row_count: 2,
            column_count: 2,
        },
    )
    .expect("window render");
    let window_texts: Vec<_> = window_page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(node) => Some(node.text.as_str()),
            _ => None,
        })
        .collect();
    assert!(window_texts.contains(&"420"));
    assert!(window_texts.contains(&"inline"));
}

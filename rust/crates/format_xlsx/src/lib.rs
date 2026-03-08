use format_shared::{parse_package_relationships, resolve_relationship_target, PackageRelationship};
use viewer_core::archive::OoxmlArchive;
use viewer_core::model::{
    BoxNode, PageRenderModel, Rect, RenderNode, SelectionAnchor, TextNode, TextRange, TextStyle,
};
use viewer_core::search::{search_pages, SearchMatch, SearchPage};
use viewer_core::xml::parse_document;
use viewer_core::ViewerError;

const OFFICE_DOCUMENT_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const SHARED_STRINGS_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings";
const STYLES_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const WORKSHEET_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XlsxWorkbook {
    pub workbook_part: String,
    pub active_sheet_index: Option<u32>,
    pub date_1904: bool,
    pub shared_strings_part: Option<String>,
    pub styles_part: Option<String>,
    pub sheets: Vec<XlsxWorksheet>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XlsxWorksheet {
    pub name: String,
    pub sheet_id: u32,
    pub relationship_id: String,
    pub part_name: String,
    pub visibility: WorksheetVisibility,
    pub dimension_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorksheetVisibility {
    Visible,
    Hidden,
    VeryHidden,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorksheetGridMetrics {
    pub part_name: String,
    pub default_row_height_points: Option<f32>,
    pub default_column_width: Option<f32>,
    pub columns: Vec<WorksheetColumnMetric>,
    pub rows: Vec<WorksheetRowMetric>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorksheetColumnMetric {
    pub min: u32,
    pub max: u32,
    pub width: Option<f32>,
    pub hidden: bool,
    pub custom_width: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorksheetRowMetric {
    pub index: u32,
    pub height_points: Option<f32>,
    pub hidden: bool,
    pub custom_height: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorksheetMergedCells {
    pub part_name: String,
    pub ranges: Vec<XlsxMergedCellRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XlsxMergedCellRange {
    pub reference: String,
    pub start_row: u32,
    pub start_column: u32,
    pub end_row: u32,
    pub end_column: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorksheetFrozenPanes {
    pub part_name: String,
    pub pane: Option<XlsxFrozenPane>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XlsxFrozenPane {
    pub x_split: Option<f32>,
    pub y_split: Option<f32>,
    pub top_left_cell: Option<String>,
    pub state: XlsxFrozenPaneState,
    pub active_pane: Option<XlsxActivePane>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XlsxFrozenPaneState {
    Frozen,
    FrozenSplit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XlsxActivePane {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XlsxStyleCatalog {
    pub part_name: Option<String>,
    pub number_formats: Vec<XlsxNumberFormat>,
    pub fonts: Vec<XlsxFontStyle>,
    pub fills: Vec<XlsxFillStyle>,
    pub cell_formats: Vec<XlsxCellFormat>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorksheetCells {
    pub part_name: String,
    pub cells: Vec<XlsxCell>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XlsxCell {
    pub reference: String,
    pub row: u32,
    pub column: u32,
    pub style_index: Option<u32>,
    pub formula: Option<String>,
    pub value: Option<XlsxCellValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum XlsxCellValue {
    Text(String),
    Number(String),
    Boolean(bool),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XlsxVisibleWindow {
    pub start_row: u32,
    pub start_column: u32,
    pub row_count: u32,
    pub column_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XlsxNumberFormat {
    pub id: u32,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XlsxFontStyle {
    pub font_name: Option<String>,
    pub font_size_points: Option<f32>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XlsxFillStyle {
    pub pattern_type: Option<String>,
    pub foreground_color_hex: Option<String>,
    pub background_color_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XlsxCellFormat {
    pub num_fmt_id: u32,
    pub number_format_code: Option<String>,
    pub font_id: u32,
    pub fill_id: u32,
    pub apply_number_format: bool,
    pub apply_alignment: bool,
    pub horizontal_alignment: Option<XlsxHorizontalAlignment>,
    pub vertical_alignment: Option<XlsxVerticalAlignment>,
    pub wrap_text: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XlsxHorizontalAlignment {
    General,
    Left,
    Center,
    Right,
    Fill,
    Justify,
    CenterContinuous,
    Distributed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XlsxVerticalAlignment {
    Top,
    Center,
    Bottom,
    Justify,
    Distributed,
}

pub fn parse_xlsx(archive: &OoxmlArchive) -> Result<XlsxWorkbook, ViewerError> {
    let package_relationships = parse_package_relationships(archive)?;
    let workbook_part = package_relationships
        .iter()
        .find(|relationship| relationship.relationship_type == OFFICE_DOCUMENT_RELATIONSHIP)
        .map(|relationship| relationship.resolved_target.trim_start_matches('/').to_string())
        .ok_or(ViewerError::InvalidDocument)?;

    let workbook_xml = archive.read_part(&workbook_part)?;
    let workbook_text = String::from_utf8(workbook_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let workbook_root = parse_document(&workbook_text)?;
    if workbook_root.local_name() != "workbook" {
        return Err(ViewerError::InvalidDocument);
    }

    let workbook_relationships = parse_workbook_relationships(archive, &workbook_part)?;
    let active_sheet_index = workbook_root
        .child("bookViews")
        .and_then(|book_views| book_views.child("workbookView"))
        .and_then(|view| view.attribute("activeTab"))
        .map(|value| value.parse::<u32>().map_err(|_| ViewerError::InvalidDocument))
        .transpose()?;
    let date_1904 = workbook_root
        .child("workbookPr")
        .and_then(|properties| properties.attribute("date1904"))
        .map(|value| matches!(value, "1" | "true"))
        .unwrap_or(false);
    let shared_strings_part = workbook_relationships
        .iter()
        .find(|relationship| relationship.relationship_type == SHARED_STRINGS_RELATIONSHIP)
        .map(|relationship| relationship.resolved_target.trim_start_matches('/').to_string());
    let styles_part = workbook_relationships
        .iter()
        .find(|relationship| relationship.relationship_type == STYLES_RELATIONSHIP)
        .map(|relationship| relationship.resolved_target.trim_start_matches('/').to_string());

    let sheets_root = workbook_root.child("sheets").ok_or(ViewerError::InvalidDocument)?;
    let mut sheets = Vec::new();
    for child in &sheets_root.children {
        if child.local_name() != "sheet" {
            continue;
        }

        let name = child.required_attribute("name")?.to_string();
        let relationship_id = child.required_attribute("id")?.to_string();
        let sheet_id = child
            .required_attribute("sheetId")?
            .parse::<u32>()
            .map_err(|_| ViewerError::InvalidDocument)?;
        let visibility = parse_sheet_visibility(child.attribute("state"));

        let relationship = workbook_relationships
            .iter()
            .find(|relationship| relationship.id == relationship_id)
            .ok_or(ViewerError::InvalidDocument)?;
        if relationship.relationship_type != WORKSHEET_RELATIONSHIP {
            return Err(ViewerError::InvalidDocument);
        }

        let part_name = relationship.resolved_target.trim_start_matches('/').to_string();
        let dimension_ref = parse_worksheet_dimension(archive, &part_name)?;

        sheets.push(XlsxWorksheet {
            name,
            sheet_id,
            relationship_id,
            part_name,
            visibility,
            dimension_ref,
        });
    }

    Ok(XlsxWorkbook {
        workbook_part,
        active_sheet_index,
        date_1904,
        shared_strings_part,
        styles_part,
        sheets,
    })
}

pub fn parse_shared_strings(
    archive: &OoxmlArchive,
    workbook: &XlsxWorkbook,
) -> Result<Vec<String>, ViewerError> {
    let Some(part_name) = workbook.shared_strings_part.as_deref() else {
        return Ok(Vec::new());
    };

    let xml = archive.read_part(part_name)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;
    if root.local_name() != "sst" {
        return Err(ViewerError::InvalidDocument);
    }

    let mut strings = Vec::new();
    for child in &root.children {
        if child.local_name() != "si" {
            continue;
        }
        strings.push(parse_shared_string_item(child));
    }

    Ok(strings)
}

pub fn parse_row_column_metrics(
    archive: &OoxmlArchive,
    workbook: &XlsxWorkbook,
) -> Result<Vec<WorksheetGridMetrics>, ViewerError> {
    workbook
        .sheets
        .iter()
        .map(|sheet| parse_worksheet_grid_metrics(archive, &sheet.part_name))
        .collect()
}

pub fn parse_cell_style_subset(
    archive: &OoxmlArchive,
    workbook: &XlsxWorkbook,
) -> Result<XlsxStyleCatalog, ViewerError> {
    let Some(part_name) = workbook.styles_part.as_deref() else {
        return Ok(XlsxStyleCatalog {
            part_name: None,
            number_formats: Vec::new(),
            fonts: Vec::new(),
            fills: Vec::new(),
            cell_formats: Vec::new(),
        });
    };

    let xml = archive.read_part(part_name)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;
    if root.local_name() != "styleSheet" {
        return Err(ViewerError::InvalidDocument);
    }

    let number_formats = root
        .child("numFmts")
        .map(parse_number_formats)
        .transpose()?
        .unwrap_or_default();
    let fonts = root
        .child("fonts")
        .map(parse_fonts)
        .transpose()?
        .unwrap_or_default();
    let fills = root
        .child("fills")
        .map(parse_fills)
        .transpose()?
        .unwrap_or_default();
    let cell_formats = root
        .child("cellXfs")
        .map(|cell_xfs| parse_cell_formats(cell_xfs, &number_formats))
        .transpose()?
        .unwrap_or_default();

    Ok(XlsxStyleCatalog {
        part_name: Some(part_name.to_string()),
        number_formats,
        fonts,
        fills,
        cell_formats,
    })
}

pub fn parse_merged_cells(
    archive: &OoxmlArchive,
    workbook: &XlsxWorkbook,
) -> Result<Vec<WorksheetMergedCells>, ViewerError> {
    workbook
        .sheets
        .iter()
        .map(|sheet| parse_worksheet_merged_cells(archive, &sheet.part_name))
        .collect()
}

pub fn parse_frozen_panes(
    archive: &OoxmlArchive,
    workbook: &XlsxWorkbook,
) -> Result<Vec<WorksheetFrozenPanes>, ViewerError> {
    workbook
        .sheets
        .iter()
        .map(|sheet| parse_worksheet_frozen_panes(archive, &sheet.part_name))
        .collect()
}

pub fn parse_worksheet_cells(
    archive: &OoxmlArchive,
    workbook: &XlsxWorkbook,
    shared_strings: &[String],
) -> Result<Vec<WorksheetCells>, ViewerError> {
    workbook
        .sheets
        .iter()
        .map(|sheet| parse_cells_for_worksheet(archive, &sheet.part_name, shared_strings))
        .collect()
}

pub fn build_sheet_render_model(
    archive: &OoxmlArchive,
    workbook: &XlsxWorkbook,
    shared_strings: &[String],
    styles: &XlsxStyleCatalog,
    sheet_index: usize,
) -> Result<PageRenderModel, ViewerError> {
    let sheet = workbook
        .sheets
        .get(sheet_index)
        .ok_or(ViewerError::InvalidDocument)?;
    let cells_by_sheet = parse_worksheet_cells(archive, workbook, shared_strings)?;
    let metrics_by_sheet = parse_row_column_metrics(archive, workbook)?;
    let merges_by_sheet = parse_merged_cells(archive, workbook)?;

    let worksheet_cells = cells_by_sheet
        .iter()
        .find(|worksheet| worksheet.part_name == sheet.part_name)
        .ok_or(ViewerError::InvalidDocument)?;
    let metrics = metrics_by_sheet
        .iter()
        .find(|worksheet| worksheet.part_name == sheet.part_name)
        .ok_or(ViewerError::InvalidDocument)?;
    let merges = merges_by_sheet
        .iter()
        .find(|worksheet| worksheet.part_name == sheet.part_name)
        .ok_or(ViewerError::InvalidDocument)?;

    let full_bounds = sheet
        .dimension_ref
        .as_deref()
        .map(parse_dimension_reference)
        .transpose()?
        .unwrap_or_else(|| infer_sheet_bounds(worksheet_cells, merges));

    render_sheet_model(
        sheet_index as u32,
        worksheet_cells,
        metrics,
        merges,
        styles,
        full_bounds,
    )
}

pub fn build_visible_window_render_model(
    archive: &OoxmlArchive,
    workbook: &XlsxWorkbook,
    shared_strings: &[String],
    styles: &XlsxStyleCatalog,
    sheet_index: usize,
    window: &XlsxVisibleWindow,
) -> Result<PageRenderModel, ViewerError> {
    if window.row_count == 0 || window.column_count == 0 {
        return Err(ViewerError::InvalidDocument);
    }

    let sheet = workbook
        .sheets
        .get(sheet_index)
        .ok_or(ViewerError::InvalidDocument)?;
    let cells_by_sheet = parse_worksheet_cells(archive, workbook, shared_strings)?;
    let metrics_by_sheet = parse_row_column_metrics(archive, workbook)?;
    let merges_by_sheet = parse_merged_cells(archive, workbook)?;

    let worksheet_cells = cells_by_sheet
        .iter()
        .find(|worksheet| worksheet.part_name == sheet.part_name)
        .ok_or(ViewerError::InvalidDocument)?;
    let metrics = metrics_by_sheet
        .iter()
        .find(|worksheet| worksheet.part_name == sheet.part_name)
        .ok_or(ViewerError::InvalidDocument)?;
    let merges = merges_by_sheet
        .iter()
        .find(|worksheet| worksheet.part_name == sheet.part_name)
        .ok_or(ViewerError::InvalidDocument)?;
    let (sheet_start_row, sheet_start_column, sheet_end_row, sheet_end_column) = sheet
        .dimension_ref
        .as_deref()
        .map(parse_dimension_reference)
        .transpose()?
        .unwrap_or_else(|| infer_sheet_bounds(worksheet_cells, merges));
    let render_start_row = window.start_row.max(sheet_start_row);
    let render_start_column = window.start_column.max(sheet_start_column);
    let render_end_row = (window.start_row + window.row_count - 1).min(sheet_end_row);
    let render_end_column = (window.start_column + window.column_count - 1).min(sheet_end_column);
    if render_start_row > render_end_row || render_start_column > render_end_column {
        return Err(ViewerError::InvalidDocument);
    }

    render_sheet_model(
        sheet_index as u32,
        worksheet_cells,
        metrics,
        merges,
        styles,
        (
            render_start_row,
            render_start_column,
            render_end_row,
            render_end_column,
        ),
    )
}

pub fn build_search_pages(
    archive: &OoxmlArchive,
    workbook: &XlsxWorkbook,
    shared_strings: &[String],
) -> Result<Vec<SearchPage>, ViewerError> {
    let worksheet_cells = parse_worksheet_cells(archive, workbook, shared_strings)?;
    let mut pages = Vec::new();

    for (sheet_index, sheet) in workbook.sheets.iter().enumerate() {
        let worksheet = worksheet_cells
            .iter()
            .find(|worksheet| worksheet.part_name == sheet.part_name)
            .ok_or(ViewerError::InvalidDocument)?;
        let mut sorted_cells = worksheet.cells.iter().collect::<Vec<_>>();
        sorted_cells.sort_by_key(|cell| (cell.row, cell.column));

        let mut current_row = None;
        let mut row_values = Vec::new();
        let mut lines = Vec::new();
        for cell in sorted_cells {
            if current_row != Some(cell.row) {
                if !row_values.is_empty() {
                    lines.push(row_values.join("\t"));
                    row_values.clear();
                }
                current_row = Some(cell.row);
            }

            if let Some(display_text) = format_cell_display_value(&cell.value) {
                if !display_text.trim().is_empty() {
                    row_values.push(display_text);
                }
            }
        }
        if !row_values.is_empty() {
            lines.push(row_values.join("\t"));
        }

        pages.push(SearchPage {
            page_index: sheet_index as u32,
            text: lines.join("\n"),
        });
    }

    Ok(pages)
}

pub fn search_workbook(
    archive: &OoxmlArchive,
    workbook: &XlsxWorkbook,
    shared_strings: &[String],
    query: &str,
) -> Result<Vec<SearchMatch>, ViewerError> {
    let pages = build_search_pages(archive, workbook, shared_strings)?;
    Ok(search_pages(&pages, query))
}

fn parse_workbook_relationships(
    archive: &OoxmlArchive,
    workbook_part: &str,
) -> Result<Vec<PackageRelationship>, ViewerError> {
    let rels_part = workbook_relationship_part(workbook_part)?;
    let rels_xml = archive.read_part(&rels_part)?;
    let rels_text = String::from_utf8(rels_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let rels_root = parse_document(&rels_text)?;

    let mut relationships = Vec::new();
    for child in &rels_root.children {
        if child.local_name() != "Relationship" {
            continue;
        }

        let target = child.required_attribute("Target")?.to_string();
        if child.attribute("TargetMode") == Some("External") {
            return Err(ViewerError::InvalidDocument);
        }

        let resolved_target = resolve_relationship_target(&format!("/{workbook_part}"), &target);
        if !archive.contains_part(resolved_target.trim_start_matches('/')) {
            return Err(ViewerError::InvalidDocument);
        }

        relationships.push(PackageRelationship {
            id: child.required_attribute("Id")?.to_string(),
            relationship_type: child.required_attribute("Type")?.to_string(),
            target,
            resolved_target,
        });
    }

    Ok(relationships)
}

fn workbook_relationship_part(workbook_part: &str) -> Result<String, ViewerError> {
    let slash_index = workbook_part.rfind('/').ok_or(ViewerError::InvalidDocument)?;
    let (directory, file_name) = workbook_part.split_at(slash_index + 1);
    Ok(format!("{directory}_rels/{file_name}.rels"))
}

fn parse_worksheet_dimension(
    archive: &OoxmlArchive,
    worksheet_part: &str,
) -> Result<Option<String>, ViewerError> {
    let worksheet_xml = archive.read_part(worksheet_part)?;
    let worksheet_text =
        String::from_utf8(worksheet_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let worksheet_root = parse_document(&worksheet_text)?;
    if worksheet_root.local_name() != "worksheet" {
        return Err(ViewerError::InvalidDocument);
    }

    Ok(worksheet_root
        .child("dimension")
        .and_then(|dimension| dimension.attribute("ref"))
        .map(ToOwned::to_owned))
}

fn parse_worksheet_grid_metrics(
    archive: &OoxmlArchive,
    worksheet_part: &str,
) -> Result<WorksheetGridMetrics, ViewerError> {
    let worksheet_xml = archive.read_part(worksheet_part)?;
    let worksheet_text =
        String::from_utf8(worksheet_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let worksheet_root = parse_document(&worksheet_text)?;
    if worksheet_root.local_name() != "worksheet" {
        return Err(ViewerError::InvalidDocument);
    }

    let (default_row_height_points, default_column_width) = worksheet_root
        .child("sheetFormatPr")
        .map(|format| {
            let default_row_height_points = format
                .attribute("defaultRowHeight")
                .map(parse_decimal)
                .transpose()?;
            let default_column_width = format
                .attribute("defaultColWidth")
                .map(parse_decimal)
                .transpose()?;
            Ok((default_row_height_points, default_column_width))
        })
        .transpose()?
        .unwrap_or((None, None));

    let mut columns = Vec::new();
    if let Some(cols) = worksheet_root.child("cols") {
        for column in &cols.children {
            if column.local_name() != "col" {
                continue;
            }

            columns.push(WorksheetColumnMetric {
                min: column
                    .required_attribute("min")?
                    .parse::<u32>()
                    .map_err(|_| ViewerError::InvalidDocument)?,
                max: column
                    .required_attribute("max")?
                    .parse::<u32>()
                    .map_err(|_| ViewerError::InvalidDocument)?,
                width: column
                    .attribute("width")
                    .map(parse_decimal)
                    .transpose()?,
                hidden: matches!(column.attribute("hidden"), Some("1" | "true")),
                custom_width: matches!(column.attribute("customWidth"), Some("1" | "true")),
            });
        }
    }

    let mut rows = Vec::new();
    if let Some(sheet_data) = worksheet_root.child("sheetData") {
        for row in &sheet_data.children {
            if row.local_name() != "row" {
                continue;
            }

            rows.push(WorksheetRowMetric {
                index: row
                    .required_attribute("r")?
                    .parse::<u32>()
                    .map_err(|_| ViewerError::InvalidDocument)?,
                height_points: row
                    .attribute("ht")
                    .map(parse_decimal)
                    .transpose()?,
                hidden: matches!(row.attribute("hidden"), Some("1" | "true")),
                custom_height: matches!(row.attribute("customHeight"), Some("1" | "true")),
            });
        }
    }

    Ok(WorksheetGridMetrics {
        part_name: worksheet_part.to_string(),
        default_row_height_points,
        default_column_width,
        columns,
        rows,
    })
}

fn parse_worksheet_merged_cells(
    archive: &OoxmlArchive,
    worksheet_part: &str,
) -> Result<WorksheetMergedCells, ViewerError> {
    let worksheet_xml = archive.read_part(worksheet_part)?;
    let worksheet_text =
        String::from_utf8(worksheet_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let worksheet_root = parse_document(&worksheet_text)?;
    if worksheet_root.local_name() != "worksheet" {
        return Err(ViewerError::InvalidDocument);
    }

    let mut ranges = Vec::new();
    if let Some(merge_cells) = worksheet_root.child("mergeCells") {
        for merge_cell in &merge_cells.children {
            if merge_cell.local_name() != "mergeCell" {
                continue;
            }

            let reference = merge_cell.required_attribute("ref")?.to_string();
            let (start_row, start_column, end_row, end_column) =
                parse_merged_cell_reference(&reference)?;
            ranges.push(XlsxMergedCellRange {
                reference,
                start_row,
                start_column,
                end_row,
                end_column,
            });
        }
    }

    Ok(WorksheetMergedCells {
        part_name: worksheet_part.to_string(),
        ranges,
    })
}

fn parse_worksheet_frozen_panes(
    archive: &OoxmlArchive,
    worksheet_part: &str,
) -> Result<WorksheetFrozenPanes, ViewerError> {
    let worksheet_xml = archive.read_part(worksheet_part)?;
    let worksheet_text =
        String::from_utf8(worksheet_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let worksheet_root = parse_document(&worksheet_text)?;
    if worksheet_root.local_name() != "worksheet" {
        return Err(ViewerError::InvalidDocument);
    }

    let pane = worksheet_root
        .child("sheetViews")
        .and_then(|sheet_views| sheet_views.child("sheetView"))
        .and_then(|sheet_view| sheet_view.child("pane"))
        .map(parse_frozen_pane)
        .transpose()?
        .flatten();

    Ok(WorksheetFrozenPanes {
        part_name: worksheet_part.to_string(),
        pane,
    })
}

fn parse_cells_for_worksheet(
    archive: &OoxmlArchive,
    worksheet_part: &str,
    shared_strings: &[String],
) -> Result<WorksheetCells, ViewerError> {
    let worksheet_xml = archive.read_part(worksheet_part)?;
    let worksheet_text =
        String::from_utf8(worksheet_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let worksheet_root = parse_document(&worksheet_text)?;
    if worksheet_root.local_name() != "worksheet" {
        return Err(ViewerError::InvalidDocument);
    }

    let mut cells = Vec::new();
    if let Some(sheet_data) = worksheet_root.child("sheetData") {
        for row in &sheet_data.children {
            if row.local_name() != "row" {
                continue;
            }

            for cell in &row.children {
                if cell.local_name() != "c" {
                    continue;
                }

                cells.push(parse_cell(cell, shared_strings)?);
            }
        }
    }

    Ok(WorksheetCells {
        part_name: worksheet_part.to_string(),
        cells,
    })
}

fn render_sheet_model(
    sheet_index: u32,
    worksheet_cells: &WorksheetCells,
    metrics: &WorksheetGridMetrics,
    merges: &WorksheetMergedCells,
    styles: &XlsxStyleCatalog,
    render_bounds: (u32, u32, u32, u32),
) -> Result<PageRenderModel, ViewerError> {
    let (start_row, start_column, end_row, end_column) = render_bounds;

    let column_widths: Vec<f32> = (start_column..=end_column)
        .map(|column| resolve_column_width(metrics, column))
        .collect();
    let row_heights: Vec<f32> = (start_row..=end_row)
        .map(|row| resolve_row_height(metrics, row))
        .collect();
    let width = column_widths.iter().sum::<f32>();
    let height = row_heights.iter().sum::<f32>();

    let mut nodes = Vec::new();
    let mut selection_anchors = Vec::new();
    let mut selection_offset = 0u32;

    for row in start_row..=end_row {
        let cell_y = sum_lengths(&row_heights, start_row, row);
        for column in start_column..=end_column {
            if merges
                .ranges
                .iter()
                .any(|range| is_covered_by_merged_range(range, row, column) && !is_merge_origin(range, row, column))
            {
                continue;
            }

            let cell_x = sum_lengths(&column_widths, start_column, column);
            let cell = worksheet_cells
                .cells
                .iter()
                .find(|cell| cell.row == row && cell.column == column);
            let merged_range = merges
                .ranges
                .iter()
                .find(|range| is_merge_origin(range, row, column));
            let span_columns = merged_range
                .map(|range| range.end_column - range.start_column + 1)
                .unwrap_or(1);
            let span_rows = merged_range
                .map(|range| range.end_row - range.start_row + 1)
                .unwrap_or(1);
            let cell_width = column_span_width(&column_widths, start_column, column, span_columns);
            let cell_height = row_span_height(&row_heights, start_row, row, span_rows);
            let cell_rect = Rect {
                x: cell_x,
                y: cell_y,
                width: cell_width,
                height: cell_height,
            };
            let style = cell
                .and_then(|cell| cell.style_index)
                .and_then(|index| styles.cell_formats.get(index as usize));
            let fill_color_hex = style
                .and_then(|format| styles.fills.get(format.fill_id as usize))
                .and_then(|fill| fill.foreground_color_hex.clone().or(fill.background_color_hex.clone()));

            nodes.push(RenderNode::Box(BoxNode {
                bounds: cell_rect.clone(),
                fill_color_hex,
                gradient_end_color_hex: None,
                gradient_angle_degrees: None,
                stroke_color_hex: Some("#D0D7DE".to_string()),
                stroke_width: 1.0,
                corner_radius: None,
            }));

            if let Some(cell) = cell {
                if let Some(display_text) = format_cell_display_value(&cell.value) {
                    if !display_text.is_empty() {
                        let text_style = resolve_text_style(styles, style);
                        let text_bounds =
                            layout_cell_text(&display_text, &cell_rect, style, &text_style);
                        let range_start = selection_offset;
                        let range_end = range_start + display_text.chars().count() as u32;
                        let node_index = nodes.len() as u32;
                        selection_anchors.extend(build_text_selection_anchors(
                            &display_text,
                            &text_bounds,
                            &text_style,
                            node_index,
                        ));
                        selection_offset = range_end;

                        nodes.push(RenderNode::Text(TextNode {
                            text: display_text,
                            bounds: text_bounds,
                            style: text_style,
                            range: TextRange {
                                start: range_start,
                                end: range_end,
                            },
                        }));
                    }
                }
            }
        }
    }

    Ok(PageRenderModel {
        page_index: sheet_index,
        width,
        height,
        nodes,
        selection_anchors,
    })
}

fn parse_sheet_visibility(state: Option<&str>) -> WorksheetVisibility {
    match state {
        Some("hidden") => WorksheetVisibility::Hidden,
        Some("veryHidden") => WorksheetVisibility::VeryHidden,
        _ => WorksheetVisibility::Visible,
    }
}

fn parse_shared_string_item(item: &viewer_core::xml::XmlElement) -> String {
    let mut text = String::new();
    for child in &item.children {
        match child.local_name() {
            "t" => text.push_str(&child.text),
            "r" => {
                if let Some(run_text) = child.child("t") {
                    text.push_str(&run_text.text);
                }
            }
            _ => {}
        }
    }
    text
}

fn parse_decimal(value: &str) -> Result<f32, ViewerError> {
    value.parse::<f32>().map_err(|_| ViewerError::InvalidDocument)
}

fn parse_number_formats(
    num_fmts: &viewer_core::xml::XmlElement,
) -> Result<Vec<XlsxNumberFormat>, ViewerError> {
    let mut formats = Vec::new();
    for child in &num_fmts.children {
        if child.local_name() != "numFmt" {
            continue;
        }

        formats.push(XlsxNumberFormat {
            id: child
                .required_attribute("numFmtId")?
                .parse::<u32>()
                .map_err(|_| ViewerError::InvalidDocument)?,
            code: child.required_attribute("formatCode")?.to_string(),
        });
    }
    Ok(formats)
}

fn parse_fonts(
    fonts: &viewer_core::xml::XmlElement,
) -> Result<Vec<XlsxFontStyle>, ViewerError> {
    let mut parsed_fonts = Vec::new();
    for child in &fonts.children {
        if child.local_name() != "font" {
            continue;
        }

        parsed_fonts.push(XlsxFontStyle {
            font_name: child
                .child("name")
                .and_then(|name| name.attribute("val"))
                .map(ToOwned::to_owned),
            font_size_points: child
                .child("sz")
                .and_then(|size| size.attribute("val"))
                .map(parse_decimal)
                .transpose()?,
            bold: child.child("b").is_some(),
            italic: child.child("i").is_some(),
            underline: child.child("u").is_some(),
            color_hex: child.child("color").and_then(parse_xlsx_color),
        });
    }
    Ok(parsed_fonts)
}

fn parse_fills(
    fills: &viewer_core::xml::XmlElement,
) -> Result<Vec<XlsxFillStyle>, ViewerError> {
    let mut parsed_fills = Vec::new();
    for child in &fills.children {
        if child.local_name() != "fill" {
            continue;
        }

        let pattern_fill = child.child("patternFill");
        parsed_fills.push(XlsxFillStyle {
            pattern_type: pattern_fill
                .and_then(|fill| fill.attribute("patternType"))
                .map(ToOwned::to_owned),
            foreground_color_hex: pattern_fill
                .and_then(|fill| fill.child("fgColor"))
                .and_then(parse_xlsx_color),
            background_color_hex: pattern_fill
                .and_then(|fill| fill.child("bgColor"))
                .and_then(parse_xlsx_color),
        });
    }
    Ok(parsed_fills)
}

fn parse_cell_formats(
    cell_xfs: &viewer_core::xml::XmlElement,
    number_formats: &[XlsxNumberFormat],
) -> Result<Vec<XlsxCellFormat>, ViewerError> {
    let mut formats = Vec::new();
    for child in &cell_xfs.children {
        if child.local_name() != "xf" {
            continue;
        }

        let num_fmt_id = child
            .required_attribute("numFmtId")?
            .parse::<u32>()
            .map_err(|_| ViewerError::InvalidDocument)?;
        let font_id = child
            .required_attribute("fontId")?
            .parse::<u32>()
            .map_err(|_| ViewerError::InvalidDocument)?;
        let fill_id = child
            .required_attribute("fillId")?
            .parse::<u32>()
            .map_err(|_| ViewerError::InvalidDocument)?;
        let alignment = child.child("alignment");

        formats.push(XlsxCellFormat {
            num_fmt_id,
            number_format_code: number_formats
                .iter()
                .find(|format| format.id == num_fmt_id)
                .map(|format| format.code.clone()),
            font_id,
            fill_id,
            apply_number_format: matches!(
                child.attribute("applyNumberFormat"),
                Some("1" | "true")
            ),
            apply_alignment: matches!(child.attribute("applyAlignment"), Some("1" | "true")),
            horizontal_alignment: alignment
                .and_then(|alignment| alignment.attribute("horizontal"))
                .map(parse_horizontal_alignment)
                .transpose()?,
            vertical_alignment: alignment
                .and_then(|alignment| alignment.attribute("vertical"))
                .map(parse_vertical_alignment)
                .transpose()?,
            wrap_text: matches!(
                alignment.and_then(|alignment| alignment.attribute("wrapText")),
                Some("1" | "true")
            ),
        });
    }
    Ok(formats)
}

fn parse_horizontal_alignment(value: &str) -> Result<XlsxHorizontalAlignment, ViewerError> {
    match value {
        "general" => Ok(XlsxHorizontalAlignment::General),
        "left" => Ok(XlsxHorizontalAlignment::Left),
        "center" => Ok(XlsxHorizontalAlignment::Center),
        "right" => Ok(XlsxHorizontalAlignment::Right),
        "fill" => Ok(XlsxHorizontalAlignment::Fill),
        "justify" => Ok(XlsxHorizontalAlignment::Justify),
        "centerContinuous" => Ok(XlsxHorizontalAlignment::CenterContinuous),
        "distributed" => Ok(XlsxHorizontalAlignment::Distributed),
        _ => Err(ViewerError::InvalidDocument),
    }
}

fn parse_vertical_alignment(value: &str) -> Result<XlsxVerticalAlignment, ViewerError> {
    match value {
        "top" => Ok(XlsxVerticalAlignment::Top),
        "center" => Ok(XlsxVerticalAlignment::Center),
        "bottom" => Ok(XlsxVerticalAlignment::Bottom),
        "justify" => Ok(XlsxVerticalAlignment::Justify),
        "distributed" => Ok(XlsxVerticalAlignment::Distributed),
        _ => Err(ViewerError::InvalidDocument),
    }
}

fn parse_xlsx_color(color: &viewer_core::xml::XmlElement) -> Option<String> {
    if let Some(rgb) = color.attribute("rgb") {
        return normalize_argb_hex(rgb);
    }
    None
}

fn parse_frozen_pane(
    pane: &viewer_core::xml::XmlElement,
) -> Result<Option<XlsxFrozenPane>, ViewerError> {
    let Some(state) = pane.attribute("state") else {
        return Ok(None);
    };

    let state = match state {
        "frozen" => XlsxFrozenPaneState::Frozen,
        "frozenSplit" => XlsxFrozenPaneState::FrozenSplit,
        "split" => return Ok(None),
        _ => return Err(ViewerError::InvalidDocument),
    };

    let top_left_cell = pane.attribute("topLeftCell").map(ToOwned::to_owned);
    if let Some(reference) = top_left_cell.as_deref() {
        parse_cell_reference(reference)?;
    }

    Ok(Some(XlsxFrozenPane {
        x_split: pane.attribute("xSplit").map(parse_decimal).transpose()?,
        y_split: pane.attribute("ySplit").map(parse_decimal).transpose()?,
        top_left_cell,
        state,
        active_pane: pane
            .attribute("activePane")
            .map(parse_active_pane)
            .transpose()?,
    }))
}

fn parse_active_pane(value: &str) -> Result<XlsxActivePane, ViewerError> {
    match value {
        "topLeft" => Ok(XlsxActivePane::TopLeft),
        "topRight" => Ok(XlsxActivePane::TopRight),
        "bottomLeft" => Ok(XlsxActivePane::BottomLeft),
        "bottomRight" => Ok(XlsxActivePane::BottomRight),
        _ => Err(ViewerError::InvalidDocument),
    }
}

fn parse_cell(
    cell: &viewer_core::xml::XmlElement,
    shared_strings: &[String],
) -> Result<XlsxCell, ViewerError> {
    let reference = cell.required_attribute("r")?.to_string();
    let (row, column) = parse_cell_reference(&reference)?;
    let style_index = cell
        .attribute("s")
        .map(|value| value.parse::<u32>().map_err(|_| ViewerError::InvalidDocument))
        .transpose()?;
    let formula = cell.child("f").map(|formula| formula.text.clone());
    let value = parse_cell_value(cell, shared_strings)?;

    Ok(XlsxCell {
        reference,
        row,
        column,
        style_index,
        formula,
        value,
    })
}

fn parse_cell_value(
    cell: &viewer_core::xml::XmlElement,
    shared_strings: &[String],
) -> Result<Option<XlsxCellValue>, ViewerError> {
    let cell_type = cell.attribute("t");
    match cell_type {
        Some("s") => {
            let shared_index = cell
                .child("v")
                .map(|value| value.text.parse::<usize>().map_err(|_| ViewerError::InvalidDocument))
                .transpose()?
                .ok_or(ViewerError::InvalidDocument)?;
            let value = shared_strings
                .get(shared_index)
                .cloned()
                .ok_or(ViewerError::InvalidDocument)?;
            Ok(Some(XlsxCellValue::Text(value)))
        }
        Some("inlineStr") => Ok(cell
            .child("is")
            .map(parse_shared_string_item)
            .map(XlsxCellValue::Text)),
        Some("str") => Ok(cell
            .child("v")
            .map(|value| XlsxCellValue::Text(value.text.clone()))),
        Some("b") => Ok(cell
            .child("v")
            .map(|value| parse_boolean_cell_value(&value.text))
            .transpose()?
            .map(XlsxCellValue::Boolean)),
        Some("e") => Ok(cell
            .child("v")
            .map(|value| XlsxCellValue::Error(value.text.clone()))),
        Some("n") | None => Ok(cell
            .child("v")
            .map(|value| XlsxCellValue::Number(value.text.clone()))),
        _ => Err(ViewerError::InvalidDocument),
    }
}

fn parse_boolean_cell_value(value: &str) -> Result<bool, ViewerError> {
    match value {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(ViewerError::InvalidDocument),
    }
}

fn parse_dimension_reference(reference: &str) -> Result<(u32, u32, u32, u32), ViewerError> {
    if reference.contains(':') {
        return parse_merged_cell_reference(reference);
    }

    let (row, column) = parse_cell_reference(reference)?;
    Ok((row, column, row, column))
}

fn infer_sheet_bounds(
    worksheet_cells: &WorksheetCells,
    merges: &WorksheetMergedCells,
) -> (u32, u32, u32, u32) {
    let mut start_row = u32::MAX;
    let mut start_column = u32::MAX;
    let mut end_row = 1u32;
    let mut end_column = 1u32;

    for cell in &worksheet_cells.cells {
        start_row = start_row.min(cell.row);
        start_column = start_column.min(cell.column);
        end_row = end_row.max(cell.row);
        end_column = end_column.max(cell.column);
    }

    for range in &merges.ranges {
        start_row = start_row.min(range.start_row);
        start_column = start_column.min(range.start_column);
        end_row = end_row.max(range.end_row);
        end_column = end_column.max(range.end_column);
    }

    if start_row == u32::MAX || start_column == u32::MAX {
        (1, 1, 1, 1)
    } else {
        (start_row, start_column, end_row, end_column)
    }
}

fn resolve_column_width(metrics: &WorksheetGridMetrics, column: u32) -> f32 {
    let width_units = metrics
        .columns
        .iter()
        .find(|metric| column >= metric.min && column <= metric.max)
        .and_then(|metric| metric.width)
        .or(metrics.default_column_width)
        .unwrap_or(8.43);
    (width_units * 7.0).max(24.0)
}

fn resolve_row_height(metrics: &WorksheetGridMetrics, row: u32) -> f32 {
    let points = metrics
        .rows
        .iter()
        .find(|metric| metric.index == row)
        .and_then(|metric| metric.height_points)
        .or(metrics.default_row_height_points)
        .unwrap_or(15.0);
    (points * (96.0 / 72.0)).max(20.0)
}

fn sum_lengths(lengths: &[f32], start_index: u32, current_index: u32) -> f32 {
    if current_index <= start_index {
        return 0.0;
    }
    lengths
        .iter()
        .take((current_index - start_index) as usize)
        .sum()
}

fn column_span_width(
    column_widths: &[f32],
    start_column: u32,
    column: u32,
    span_columns: u32,
) -> f32 {
    let start_index = (column - start_column) as usize;
    column_widths
        .iter()
        .skip(start_index)
        .take(span_columns as usize)
        .sum()
}

fn row_span_height(row_heights: &[f32], start_row: u32, row: u32, span_rows: u32) -> f32 {
    let start_index = (row - start_row) as usize;
    row_heights
        .iter()
        .skip(start_index)
        .take(span_rows as usize)
        .sum()
}

fn is_merge_origin(range: &XlsxMergedCellRange, row: u32, column: u32) -> bool {
    range.start_row == row && range.start_column == column
}

fn is_covered_by_merged_range(range: &XlsxMergedCellRange, row: u32, column: u32) -> bool {
    row >= range.start_row
        && row <= range.end_row
        && column >= range.start_column
        && column <= range.end_column
}

fn resolve_text_style(
    styles: &XlsxStyleCatalog,
    format: Option<&XlsxCellFormat>,
) -> TextStyle {
    let font = format
        .and_then(|format| styles.fonts.get(format.font_id as usize));
    TextStyle {
        font_family: font
            .and_then(|font| font.font_name.clone())
            .unwrap_or_else(|| "Calibri".to_string()),
        font_size: font
            .and_then(|font| font.font_size_points)
            .unwrap_or(11.0),
        bold: font.map(|font| font.bold).unwrap_or(false),
        italic: font.map(|font| font.italic).unwrap_or(false),
        underline: font.map(|font| font.underline).unwrap_or(false),
        color_hex: font
            .and_then(|font| font.color_hex.clone())
            .unwrap_or_else(|| "#000000".to_string()),
        gradient_end_color_hex: None,
        gradient_angle_degrees: None,
    }
}

fn layout_cell_text(
    display_text: &str,
    cell_rect: &Rect,
    format: Option<&XlsxCellFormat>,
    text_style: &TextStyle,
) -> Rect {
    let padding_x = 4.0;
    let padding_y = 2.0;
    let available_width = (cell_rect.width - padding_x * 2.0).max(0.0);
    let estimated_width =
        estimate_text_width(display_text, text_style.font_size).min(available_width);
    let text_height = text_style.font_size * 1.2;
    let horizontal_alignment = format
        .and_then(|format| format.horizontal_alignment.as_ref())
        .unwrap_or(&XlsxHorizontalAlignment::Left);
    let vertical_alignment = format
        .and_then(|format| format.vertical_alignment.as_ref())
        .unwrap_or(&XlsxVerticalAlignment::Center);
    let x = match horizontal_alignment {
        XlsxHorizontalAlignment::Center | XlsxHorizontalAlignment::CenterContinuous => {
            cell_rect.x + ((cell_rect.width - estimated_width) / 2.0).max(padding_x)
        }
        XlsxHorizontalAlignment::Right => {
            (cell_rect.x + cell_rect.width - padding_x - estimated_width).max(cell_rect.x + padding_x)
        }
        _ => cell_rect.x + padding_x,
    };
    let y = match vertical_alignment {
        XlsxVerticalAlignment::Top => cell_rect.y + padding_y,
        XlsxVerticalAlignment::Bottom => {
            (cell_rect.y + cell_rect.height - padding_y - text_height).max(cell_rect.y + padding_y)
        }
        _ => cell_rect.y + ((cell_rect.height - text_height) / 2.0).max(padding_y),
    };

    Rect {
        x,
        y,
        width: estimated_width,
        height: text_height,
    }
}

fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    text.chars().fold(0.0, |accumulator, character| {
        let width_factor = if character.is_ascii_whitespace() {
            0.35
        } else if character.is_ascii() {
            0.56
        } else {
            0.95
        };
        accumulator + (font_size * width_factor)
    })
}

fn build_text_selection_anchors(
    text: &str,
    bounds: &Rect,
    text_style: &TextStyle,
    node_index: u32,
) -> Vec<SelectionAnchor> {
    let mut anchors = Vec::new();
    let mut x = bounds.x;
    for (char_index, character) in text.chars().enumerate() {
        anchors.push(SelectionAnchor {
            node_index,
            char_index: char_index as u32,
            x,
            y: bounds.y,
        });
        let width_factor = if character.is_ascii_whitespace() {
            0.35
        } else if character.is_ascii() {
            0.56
        } else {
            0.95
        };
        x += text_style.font_size * width_factor;
    }
    anchors.push(SelectionAnchor {
        node_index,
        char_index: text.chars().count() as u32,
        x,
        y: bounds.y,
    });
    anchors
}

fn format_cell_display_value(value: &Option<XlsxCellValue>) -> Option<String> {
    match value {
        Some(XlsxCellValue::Text(text)) => Some(text.clone()),
        Some(XlsxCellValue::Number(number)) => Some(number.clone()),
        Some(XlsxCellValue::Boolean(value)) => Some(if *value { "TRUE" } else { "FALSE" }.to_string()),
        Some(XlsxCellValue::Error(error)) => Some(error.clone()),
        None => None,
    }
}

fn normalize_argb_hex(hex: &str) -> Option<String> {
    if hex.len() == 8 {
        let alpha = &hex[0..2];
        let rgb = &hex[2..8];
        if alpha.eq_ignore_ascii_case("FF") {
            Some(format!("#{rgb}"))
        } else {
            Some(format!("#{rgb}{alpha}"))
        }
    } else if hex.len() == 6 {
        Some(format!("#{hex}"))
    } else {
        None
    }
}

fn parse_merged_cell_reference(reference: &str) -> Result<(u32, u32, u32, u32), ViewerError> {
    let mut parts = reference.split(':');
    let start = parts.next().ok_or(ViewerError::InvalidDocument)?;
    let end = parts.next().ok_or(ViewerError::InvalidDocument)?;
    if parts.next().is_some() {
        return Err(ViewerError::InvalidDocument);
    }

    let (start_row, start_column) = parse_cell_reference(start)?;
    let (end_row, end_column) = parse_cell_reference(end)?;
    if start_row > end_row || start_column > end_column {
        return Err(ViewerError::InvalidDocument);
    }

    Ok((start_row, start_column, end_row, end_column))
}

fn parse_cell_reference(reference: &str) -> Result<(u32, u32), ViewerError> {
    let mut column = String::new();
    let mut row = String::new();

    for character in reference.chars() {
        if character.is_ascii_alphabetic() {
            if !row.is_empty() {
                return Err(ViewerError::InvalidDocument);
            }
            column.push(character);
        } else if character.is_ascii_digit() {
            row.push(character);
        } else {
            return Err(ViewerError::InvalidDocument);
        }
    }

    if column.is_empty() || row.is_empty() {
        return Err(ViewerError::InvalidDocument);
    }

    let column_index = column_letters_to_index(&column)?;
    let row_index = row.parse::<u32>().map_err(|_| ViewerError::InvalidDocument)?;
    if row_index == 0 {
        return Err(ViewerError::InvalidDocument);
    }

    Ok((row_index, column_index))
}

fn column_letters_to_index(column: &str) -> Result<u32, ViewerError> {
    let mut value = 0u32;
    for character in column.chars() {
        if !character.is_ascii_alphabetic() {
            return Err(ViewerError::InvalidDocument);
        }
        value = value
            .checked_mul(26)
            .and_then(|current| {
                current.checked_add((character.to_ascii_uppercase() as u32) - ('A' as u32) + 1)
            })
            .ok_or(ViewerError::InvalidDocument)?;
    }
    Ok(value)
}

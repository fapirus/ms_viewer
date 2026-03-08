use format_shared::{parse_package_relationships, resolve_relationship_target, PackageRelationship};
use viewer_core::archive::OoxmlArchive;
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
pub struct XlsxStyleCatalog {
    pub part_name: Option<String>,
    pub number_formats: Vec<XlsxNumberFormat>,
    pub fonts: Vec<XlsxFontStyle>,
    pub fills: Vec<XlsxFillStyle>,
    pub cell_formats: Vec<XlsxCellFormat>,
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

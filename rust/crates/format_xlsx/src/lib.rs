use format_shared::{parse_package_relationships, resolve_relationship_target, PackageRelationship};
use viewer_core::archive::OoxmlArchive;
use viewer_core::xml::parse_document;
use viewer_core::ViewerError;

const OFFICE_DOCUMENT_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const SHARED_STRINGS_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings";
const WORKSHEET_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XlsxWorkbook {
    pub workbook_part: String,
    pub active_sheet_index: Option<u32>,
    pub date_1904: bool,
    pub shared_strings_part: Option<String>,
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

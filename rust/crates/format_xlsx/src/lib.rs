use format_shared::{parse_package_relationships, resolve_relationship_target, PackageRelationship};
use viewer_core::archive::OoxmlArchive;
use viewer_core::xml::parse_document;
use viewer_core::ViewerError;

const OFFICE_DOCUMENT_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const WORKSHEET_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XlsxWorkbook {
    pub workbook_part: String,
    pub active_sheet_index: Option<u32>,
    pub date_1904: bool,
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
        sheets,
    })
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

fn parse_sheet_visibility(state: Option<&str>) -> WorksheetVisibility {
    match state {
        Some("hidden") => WorksheetVisibility::Hidden,
        Some("veryHidden") => WorksheetVisibility::VeryHidden,
        _ => WorksheetVisibility::Visible,
    }
}

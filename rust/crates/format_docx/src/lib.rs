use format_shared::{parse_package_relationships, resolve_relationship_target};
use viewer_core::archive::OoxmlArchive;
use viewer_core::xml::parse_document;
use viewer_core::ViewerError;

const OFFICE_DOCUMENT_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const STYLES_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const NUMBERING_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering";
const HEADER_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
const FOOTER_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
const IMAGE_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocxRelationship {
    pub id: String,
    pub relationship_type: String,
    pub target: String,
    pub resolved_target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocxPackage {
    pub main_document: String,
    pub styles: Option<String>,
    pub numbering: Option<String>,
    pub headers: Vec<String>,
    pub footers: Vec<String>,
    pub media: Vec<DocxRelationship>,
}

pub fn parse_docx(archive: &OoxmlArchive) -> Result<DocxPackage, ViewerError> {
    let relationships = parse_package_relationships(archive)?;
    let main_document = relationships
        .iter()
        .find(|relationship| relationship.relationship_type == OFFICE_DOCUMENT_REL)
        .map(|relationship| relationship.resolved_target.trim_start_matches('/').to_string())
        .ok_or(ViewerError::InvalidDocument)?;

    let document_relationships = parse_document_relationships(archive, &main_document)?;

    Ok(DocxPackage {
        main_document,
        styles: first_target(&document_relationships, STYLES_REL),
        numbering: first_target(&document_relationships, NUMBERING_REL),
        headers: collect_targets(&document_relationships, HEADER_REL),
        footers: collect_targets(&document_relationships, FOOTER_REL),
        media: document_relationships
            .into_iter()
            .filter(|relationship| relationship.relationship_type == IMAGE_REL)
            .collect(),
    })
}

fn parse_document_relationships(
    archive: &OoxmlArchive,
    main_document: &str,
) -> Result<Vec<DocxRelationship>, ViewerError> {
    let rels_part = relationship_part_for(main_document);
    if !archive.contains_part(&rels_part) {
        return Ok(Vec::new());
    }

    let xml = archive.read_part(&rels_part)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;

    let base = format!("/{}", main_document);
    let mut relationships = Vec::new();
    for child in &root.children {
        if child.local_name() != "Relationship" {
            continue;
        }

        let target = child.required_attribute("Target")?.to_string();
        let resolved_target = resolve_relationship_target(&base, &target);
        let normalized_target = resolved_target.trim_start_matches('/').to_string();

        if !archive.contains_part(&normalized_target) {
            return Err(ViewerError::InvalidDocument);
        }

        relationships.push(DocxRelationship {
            id: child.required_attribute("Id")?.to_string(),
            relationship_type: child.required_attribute("Type")?.to_string(),
            target,
            resolved_target: normalized_target,
        });
    }

    Ok(relationships)
}

fn relationship_part_for(part_name: &str) -> String {
    let (directory, file_name) = part_name
        .rsplit_once('/')
        .map(|(directory, file_name)| (directory, file_name))
        .unwrap_or(("", part_name));

    if directory.is_empty() {
        format!("_rels/{file_name}.rels")
    } else {
        format!("{directory}/_rels/{file_name}.rels")
    }
}

fn first_target(relationships: &[DocxRelationship], relationship_type: &str) -> Option<String> {
    relationships
        .iter()
        .find(|relationship| relationship.relationship_type == relationship_type)
        .map(|relationship| relationship.resolved_target.clone())
}

fn collect_targets(relationships: &[DocxRelationship], relationship_type: &str) -> Vec<String> {
    relationships
        .iter()
        .filter(|relationship| relationship.relationship_type == relationship_type)
        .map(|relationship| relationship.resolved_target.clone())
        .collect()
}

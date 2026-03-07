use format_shared::{parse_shared_package, resolve_relationship_target};
use viewer_core::archive::OoxmlArchive;
use viewer_core::xml::parse_document;
use viewer_core::ViewerError;

const OFFICE_DOCUMENT_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const SLIDE_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationSize {
    pub width_emu: u32,
    pub height_emu: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideRelationship {
    pub id: String,
    pub relationship_type: String,
    pub target: String,
    pub resolved_target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideReference {
    pub slide_id: u32,
    pub relationship_id: String,
    pub part_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PptxSlideTree {
    pub presentation_part: String,
    pub presentation_size: Option<PresentationSize>,
    pub slides: Vec<SlideReference>,
}

pub fn parse_pptx(archive: &OoxmlArchive) -> Result<PptxSlideTree, ViewerError> {
    parse_slide_tree(archive)
}

pub fn parse_slide_tree(archive: &OoxmlArchive) -> Result<PptxSlideTree, ViewerError> {
    let shared_package = parse_shared_package(archive)?;
    let presentation_part = shared_package
        .relationships
        .iter()
        .find(|relationship| relationship.relationship_type == OFFICE_DOCUMENT_RELATIONSHIP)
        .map(|relationship| relationship.resolved_target.trim_start_matches('/').to_string())
        .ok_or(ViewerError::InvalidDocument)?;

    let presentation_xml = archive.read_part(&presentation_part)?;
    let presentation_text =
        String::from_utf8(presentation_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let presentation_root = parse_document(&presentation_text)?;

    if presentation_root.local_name() != "presentation" {
        return Err(ViewerError::InvalidDocument);
    }

    let relationships = parse_part_relationships(archive, &presentation_part)?;
    let presentation_size = parse_presentation_size(&presentation_root)?;
    let slides = parse_slide_references(&presentation_root, &relationships)?;

    Ok(PptxSlideTree {
        presentation_part,
        presentation_size,
        slides,
    })
}

pub fn parse_part_relationships(
    archive: &OoxmlArchive,
    part_name: &str,
) -> Result<Vec<SlideRelationship>, ViewerError> {
    let relationship_part = relationship_part_name(part_name)?;
    let xml = archive.read_part(&relationship_part)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;

    let mut relationships = Vec::new();
    let base = format!("/{}", part_name);

    for child in &root.children {
        if child.local_name() != "Relationship" {
            continue;
        }

        let target = child.required_attribute("Target")?.to_string();
        let target_mode = child.attribute("TargetMode");
        let resolved_target = if target_mode == Some("External") {
            target.clone()
        } else {
            resolve_relationship_target(&base, &target)
        };

        if target_mode != Some("External")
            && !archive.contains_part(resolved_target.trim_start_matches('/'))
        {
            return Err(ViewerError::InvalidDocument);
        }

        relationships.push(SlideRelationship {
            id: child.required_attribute("Id")?.to_string(),
            relationship_type: child.required_attribute("Type")?.to_string(),
            target,
            resolved_target,
        });
    }

    Ok(relationships)
}

fn parse_presentation_size(
    presentation_root: &viewer_core::xml::XmlElement,
) -> Result<Option<PresentationSize>, ViewerError> {
    let Some(size) = presentation_root.child("sldSz") else {
        return Ok(None);
    };

    Ok(Some(PresentationSize {
        width_emu: parse_u32_attribute(size, "cx")?,
        height_emu: parse_u32_attribute(size, "cy")?,
    }))
}

fn parse_slide_references(
    presentation_root: &viewer_core::xml::XmlElement,
    relationships: &[SlideRelationship],
) -> Result<Vec<SlideReference>, ViewerError> {
    let Some(slide_list) = presentation_root.child("sldIdLst") else {
        return Ok(Vec::new());
    };

    let mut slides = Vec::new();
    for child in &slide_list.children {
        if child.local_name() != "sldId" {
            continue;
        }

        let relationship_id = child.required_attribute("r:id")?.to_string();
        let slide_id = parse_u32_attribute(child, "id")?;
        let relationship = relationships
            .iter()
            .find(|relationship| {
                relationship.id == relationship_id
                    && relationship.relationship_type == SLIDE_RELATIONSHIP
            })
            .ok_or(ViewerError::InvalidDocument)?;

        slides.push(SlideReference {
            slide_id,
            relationship_id,
            part_name: relationship
                .resolved_target
                .trim_start_matches('/')
                .to_string(),
        });
    }

    Ok(slides)
}

fn relationship_part_name(part_name: &str) -> Result<String, ViewerError> {
    let Some((directory, file_name)) = part_name.rsplit_once('/') else {
        return Ok(format!("_rels/{}.rels", part_name));
    };

    if file_name.is_empty() {
        return Err(ViewerError::InvalidDocument);
    }

    Ok(format!("{directory}/_rels/{file_name}.rels"))
}

fn parse_u32_attribute(
    element: &viewer_core::xml::XmlElement,
    name: &str,
) -> Result<u32, ViewerError> {
    element
        .required_attribute(name)?
        .parse::<u32>()
        .map_err(|_| ViewerError::InvalidDocument)
}

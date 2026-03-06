use format_shared::{parse_package_relationships, resolve_relationship_target};
use viewer_core::archive::OoxmlArchive;
use viewer_core::model::{Block, TextRun, TextStyle};
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

pub fn parse_paragraph_blocks(
    archive: &OoxmlArchive,
    package: &DocxPackage,
) -> Result<Vec<Block>, ViewerError> {
    let xml = archive.read_part(&package.main_document)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;
    let body = root.child("body").ok_or(ViewerError::InvalidDocument)?;

    let mut blocks = Vec::new();
    for child in &body.children {
        if child.local_name() != "p" {
            continue;
        }

        blocks.push(Block::Paragraph {
            runs: parse_paragraph_runs(child),
        });
    }

    Ok(blocks)
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

fn parse_paragraph_runs(paragraph: &viewer_core::xml::XmlElement) -> Vec<TextRun> {
    let mut runs = Vec::new();

    for child in &paragraph.children {
        if child.local_name() != "r" {
            continue;
        }

        let run = parse_run(child);
        if !run.text.is_empty() {
            runs.push(run);
        }
    }

    runs
}

fn parse_run(run: &viewer_core::xml::XmlElement) -> TextRun {
    let style = parse_run_style(run.child("rPr"));
    let mut text = String::new();

    for child in &run.children {
        match child.local_name() {
            "t" => text.push_str(&child.text),
            "br" => text.push('\n'),
            _ => {}
        }
    }

    TextRun { text, style }
}

fn parse_run_style(run_properties: Option<&viewer_core::xml::XmlElement>) -> TextStyle {
    let mut style = default_text_style();
    let Some(run_properties) = run_properties else {
        return style;
    };

    style.bold = run_properties.child("b").is_some();
    style.italic = run_properties.child("i").is_some();

    if let Some(fonts) = run_properties.child("rFonts") {
        if let Some(font_family) = fonts.attribute("ascii").or_else(|| fonts.attribute("hAnsi")) {
            style.font_family = font_family.to_string();
        }
    }

    if let Some(size) = run_properties.child("sz").and_then(|node| node.attribute("val")) {
        if let Ok(half_points) = size.parse::<f32>() {
            style.font_size = half_points / 2.0;
        }
    }

    if let Some(color) = run_properties.child("color").and_then(|node| node.attribute("val")) {
        if color.len() == 6 {
            style.color_hex = format!("#{color}");
        }
    }

    style
}

fn default_text_style() -> TextStyle {
    TextStyle {
        font_family: "Times New Roman".to_string(),
        font_size: 12.0,
        bold: false,
        italic: false,
        color_hex: "#000000".to_string(),
    }
}

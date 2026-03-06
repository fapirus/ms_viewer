use format_shared::{parse_package_relationships, resolve_relationship_target};
use viewer_core::archive::OoxmlArchive;
use viewer_core::model::{
    Block, ImageReference, ListKind, ListMarker, TableCell, TableCellMerge, TableRow, TextRun,
    TextStyle,
};
use viewer_core::xml::parse_document;
use viewer_core::ViewerError;

use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq)]
pub struct StyleCatalog {
    pub default_run_style: ResolvedTextStyle,
    pub run_styles: HashMap<String, RunStyleDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunStyleDefinition {
    pub based_on: Option<String>,
    pub style: ResolvedTextStyle,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResolvedTextStyle {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub color_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberingCatalog {
    pub abstract_numbering: HashMap<u32, HashMap<u8, ListKind>>,
    pub numbering_instances: HashMap<u32, u32>,
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
    let styles = parse_style_catalog(archive, package)?;
    let numbering = parse_numbering_catalog(archive, package)?;
    let xml = archive.read_part(&package.main_document)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;
    let body = root.child("body").ok_or(ViewerError::InvalidDocument)?;

    let mut blocks = Vec::new();
    for child in &body.children {
        match child.local_name() {
            "p" => blocks.extend(parse_paragraph_blocks_from_element(
                child,
                &styles,
                &numbering,
                package,
            )?),
            "tbl" => blocks.push(parse_table_block(child, &styles, &numbering)),
            _ => {}
        }
    }

    Ok(blocks)
}

pub fn parse_numbering_catalog(
    archive: &OoxmlArchive,
    package: &DocxPackage,
) -> Result<NumberingCatalog, ViewerError> {
    let Some(numbering_part) = &package.numbering else {
        return Ok(NumberingCatalog {
            abstract_numbering: HashMap::new(),
            numbering_instances: HashMap::new(),
        });
    };

    let xml = archive.read_part(numbering_part)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;

    let mut abstract_numbering = HashMap::new();
    let mut numbering_instances = HashMap::new();

    for child in &root.children {
        match child.local_name() {
            "abstractNum" => {
                let Some(abstract_num_id) = child
                    .attribute("abstractNumId")
                    .and_then(|value| value.parse::<u32>().ok())
                else {
                    continue;
                };

                let mut levels = HashMap::new();
                for level in &child.children {
                    if level.local_name() != "lvl" {
                        continue;
                    }
                    let Some(ilvl) = level
                        .attribute("ilvl")
                        .and_then(|value| value.parse::<u8>().ok())
                    else {
                        continue;
                    };
                    let Some(num_fmt) = level
                        .child("numFmt")
                        .and_then(|node| node.attribute("val"))
                    else {
                        continue;
                    };
                    let kind = match num_fmt {
                        "bullet" => ListKind::Bullet,
                        _ => ListKind::Decimal,
                    };
                    levels.insert(ilvl, kind);
                }
                abstract_numbering.insert(abstract_num_id, levels);
            }
            "num" => {
                let Some(num_id) = child.attribute("numId").and_then(|value| value.parse::<u32>().ok()) else {
                    continue;
                };
                let Some(abstract_num_id) = child
                    .child("abstractNumId")
                    .and_then(|node| node.attribute("val"))
                    .and_then(|value| value.parse::<u32>().ok())
                else {
                    continue;
                };
                numbering_instances.insert(num_id, abstract_num_id);
            }
            _ => {}
        }
    }

    Ok(NumberingCatalog {
        abstract_numbering,
        numbering_instances,
    })
}

pub fn parse_style_catalog(
    archive: &OoxmlArchive,
    package: &DocxPackage,
) -> Result<StyleCatalog, ViewerError> {
    let Some(styles_part) = &package.styles else {
        return Ok(StyleCatalog {
            default_run_style: ResolvedTextStyle::default(),
            run_styles: HashMap::new(),
        });
    };

    let xml = archive.read_part(styles_part)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;

    let mut default_run_style = ResolvedTextStyle::default();
    let mut run_styles = HashMap::new();

    for child in &root.children {
        match child.local_name() {
            "docDefaults" => {
                if let Some(rpr_default) = child
                    .child("rPrDefault")
                    .and_then(|node| node.child("rPr"))
                {
                    default_run_style = parse_resolved_style(Some(rpr_default));
                }
            }
            "style" if child.attribute("type") == Some("character") => {
                let Some(style_id) = child.attribute("styleId") else {
                    continue;
                };
                let based_on = child
                    .child("basedOn")
                    .and_then(|node| node.attribute("val"))
                    .map(ToString::to_string);
                let style = parse_resolved_style(child.child("rPr"));
                run_styles.insert(
                    style_id.to_string(),
                    RunStyleDefinition { based_on, style },
                );
            }
            _ => {}
        }
    }

    Ok(StyleCatalog {
        default_run_style,
        run_styles,
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

fn parse_paragraph_runs(
    paragraph: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
) -> Vec<TextRun> {
    let mut runs = Vec::new();

    for child in &paragraph.children {
        if child.local_name() != "r" {
            continue;
        }

        let run = parse_run(child, styles);
        if !run.text.is_empty() {
            runs.push(run);
        }
    }

    runs
}

fn parse_paragraph_blocks_from_element(
    paragraph: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
    numbering: &NumberingCatalog,
    package: &DocxPackage,
) -> Result<Vec<Block>, ViewerError> {
    parse_paragraph_blocks_with_media(paragraph, styles, numbering, Some(package))
}

fn parse_paragraph_blocks_with_media(
    paragraph: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
    numbering: &NumberingCatalog,
    package: Option<&DocxPackage>,
) -> Result<Vec<Block>, ViewerError> {
    let runs = parse_paragraph_runs(paragraph, styles);
    let list = parse_paragraph_list(paragraph, numbering);
    let images = if let Some(package) = package {
        parse_paragraph_images(paragraph, package)?
    } else {
        Vec::new()
    };

    let mut blocks = Vec::new();
    if !runs.is_empty() {
        blocks.push(Block::Paragraph { runs, list });
    }
    blocks.extend(images.into_iter().map(|image| Block::Image { image }));

    Ok(blocks)
}

fn parse_paragraph_block(
    paragraph: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
    numbering: &NumberingCatalog,
) -> Block {
    Block::Paragraph {
        runs: parse_paragraph_runs(paragraph, styles),
        list: parse_paragraph_list(paragraph, numbering),
    }
}

fn parse_table_block(
    table: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
    numbering: &NumberingCatalog,
) -> Block {
    let mut rows = Vec::new();

    for child in &table.children {
        if child.local_name() != "tr" {
            continue;
        }

        let mut cells = Vec::new();
        for cell in &child.children {
            if cell.local_name() != "tc" {
                continue;
            }

            cells.push(parse_table_cell(cell, styles, numbering));
        }

        rows.push(TableRow { cells });
    }

    Block::Table { rows }
}

fn parse_table_cell(
    cell: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
    numbering: &NumberingCatalog,
) -> TableCell {
    let properties = cell.child("tcPr");
    let column_span = properties
        .and_then(|node| node.child("gridSpan"))
        .and_then(|node| node.attribute("val"))
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(1);
    let row_merge = properties
        .and_then(|node| node.child("vMerge"))
        .map(|node| match node.attribute("val") {
            Some("restart") => TableCellMerge::Restart,
            _ => TableCellMerge::Continue,
        });

    let mut blocks = Vec::new();
    for child in &cell.children {
        if child.local_name() != "p" {
            continue;
        }

        blocks.push(parse_paragraph_block(child, styles, numbering));
    }

    TableCell {
        blocks,
        column_span,
        row_merge,
    }
}

fn parse_paragraph_list(
    paragraph: &viewer_core::xml::XmlElement,
    numbering: &NumberingCatalog,
) -> Option<ListMarker> {
    let num_pr = paragraph
        .child("pPr")
        .and_then(|node| node.child("numPr"))?;
    let level = num_pr
        .child("ilvl")
        .and_then(|node| node.attribute("val"))
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(0);
    let num_id = num_pr
        .child("numId")
        .and_then(|node| node.attribute("val"))
        .and_then(|value| value.parse::<u32>().ok())?;
    let abstract_num_id = *numbering.numbering_instances.get(&num_id)?;
    let kind = numbering
        .abstract_numbering
        .get(&abstract_num_id)
        .and_then(|levels| levels.get(&level).cloned())
        .unwrap_or(ListKind::Decimal);

    Some(ListMarker {
        level,
        kind,
        num_id,
    })
}

fn parse_run(run: &viewer_core::xml::XmlElement, styles: &StyleCatalog) -> TextRun {
    let style = resolve_run_style(run, styles);
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

fn parse_paragraph_images(
    paragraph: &viewer_core::xml::XmlElement,
    package: &DocxPackage,
) -> Result<Vec<ImageReference>, ViewerError> {
    let mut images = Vec::new();

    for run in &paragraph.children {
        if run.local_name() != "r" {
            continue;
        }

        for child in &run.children {
            if child.local_name() != "drawing" {
                continue;
            }

            images.push(parse_drawing_image(child, package)?);
        }
    }

    Ok(images)
}

fn parse_drawing_image(
    drawing: &viewer_core::xml::XmlElement,
    package: &DocxPackage,
) -> Result<ImageReference, ViewerError> {
    let blip = find_descendant(drawing, "blip").ok_or(ViewerError::InvalidDocument)?;
    let relationship_id = blip
        .attribute("embed")
        .or_else(|| blip.attribute("link"))
        .ok_or(ViewerError::InvalidDocument)?;
    let media = package
        .media
        .iter()
        .find(|relationship| relationship.id == relationship_id)
        .ok_or(ViewerError::InvalidDocument)?;
    let doc_pr = find_descendant(drawing, "docPr");
    let description = doc_pr
        .and_then(|node| node.attribute("descr").or_else(|| node.attribute("name")))
        .map(ToString::to_string);

    Ok(ImageReference {
        resource_id: media.resolved_target.clone(),
        description,
        content_type: infer_content_type(&media.resolved_target),
    })
}

fn find_descendant<'a>(
    element: &'a viewer_core::xml::XmlElement,
    local_name: &str,
) -> Option<&'a viewer_core::xml::XmlElement> {
    if element.local_name() == local_name {
        return Some(element);
    }

    for child in &element.children {
        if let Some(found) = find_descendant(child, local_name) {
            return Some(found);
        }
    }

    None
}

fn infer_content_type(path: &str) -> Option<String> {
    let ext = path.rsplit('.').next()?;
    let content_type = match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        _ => return None,
    };

    Some(content_type.to_string())
}

fn resolve_run_style(run: &viewer_core::xml::XmlElement, styles: &StyleCatalog) -> TextStyle {
    let mut resolved = styles.default_run_style.clone();
    let run_properties = run.child("rPr");

    if let Some(style_id) = run_properties
        .and_then(|node| node.child("rStyle"))
        .and_then(|node| node.attribute("val"))
    {
        let named_style = resolve_named_style(styles, style_id);
        merge_resolved_style(&mut resolved, &named_style);
    }

    let direct_style = parse_resolved_style(run_properties);
    merge_resolved_style(&mut resolved, &direct_style);

    materialize_text_style(&resolved)
}

fn resolve_named_style(styles: &StyleCatalog, style_id: &str) -> ResolvedTextStyle {
    let mut chain = Vec::new();
    let mut current = Some(style_id);

    while let Some(next_style_id) = current {
        let Some(style) = styles.run_styles.get(next_style_id) else {
            break;
        };
        chain.push(style.style.clone());
        current = style.based_on.as_deref();
    }

    let mut resolved = ResolvedTextStyle::default();
    for style in chain.into_iter().rev() {
        merge_resolved_style(&mut resolved, &style);
    }

    resolved
}

fn parse_resolved_style(run_properties: Option<&viewer_core::xml::XmlElement>) -> ResolvedTextStyle {
    let Some(run_properties) = run_properties else {
        return ResolvedTextStyle::default();
    };

    let font_family = run_properties
        .child("rFonts")
        .and_then(|fonts| fonts.attribute("ascii").or_else(|| fonts.attribute("hAnsi")))
        .map(ToString::to_string);
    let font_size = run_properties
        .child("sz")
        .and_then(|node| node.attribute("val"))
        .and_then(|value| value.parse::<f32>().ok())
        .map(|half_points| half_points / 2.0);
    let color_hex = run_properties
        .child("color")
        .and_then(|node| node.attribute("val"))
        .filter(|value| value.len() == 6)
        .map(|value| format!("#{value}"));

    ResolvedTextStyle {
        font_family,
        font_size,
        bold: run_properties.child("b").map(|_| true),
        italic: run_properties.child("i").map(|_| true),
        color_hex,
    }
}

fn merge_resolved_style(target: &mut ResolvedTextStyle, source: &ResolvedTextStyle) {
    if let Some(font_family) = &source.font_family {
        target.font_family = Some(font_family.clone());
    }
    if let Some(font_size) = source.font_size {
        target.font_size = Some(font_size);
    }
    if let Some(bold) = source.bold {
        target.bold = Some(bold);
    }
    if let Some(italic) = source.italic {
        target.italic = Some(italic);
    }
    if let Some(color_hex) = &source.color_hex {
        target.color_hex = Some(color_hex.clone());
    }
}

fn materialize_text_style(style: &ResolvedTextStyle) -> TextStyle {
    let fallback = default_text_style();
    TextStyle {
        font_family: style
            .font_family
            .clone()
            .unwrap_or(fallback.font_family),
        font_size: style.font_size.unwrap_or(fallback.font_size),
        bold: style.bold.unwrap_or(fallback.bold),
        italic: style.italic.unwrap_or(fallback.italic),
        color_hex: style.color_hex.clone().unwrap_or(fallback.color_hex),
    }
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

use base64::Engine;
use format_shared::{parse_package_relationships, resolve_relationship_target};
use viewer_core::archive::OoxmlArchive;
use viewer_core::model::{
    Block, BoxNode, ImageNode, ImageReference, ListKind, ListMarker, PageRenderModel, Rect,
    RenderNode, SelectionAnchor, TableCell, TableCellMerge, TableRow, TextNode, TextRange, TextRun,
    TextStyle,
};
use viewer_core::search::{search_pages, SearchMatch, SearchPage};
use viewer_core::xml::parse_document;
use viewer_core::ViewerError;

use std::collections::HashMap;

const OFFICE_DOCUMENT_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const NUMBERING_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
const FOOTER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
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
    pub default_paragraph_metrics: ParagraphMetrics,
    pub run_styles: HashMap<String, RunStyleDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphMetrics {
    pub line_height: Option<f32>,
    pub spacing_before: f32,
    pub spacing_after: f32,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderFooterKind {
    Default,
    First,
    Even,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderFooterRef {
    pub kind: HeaderFooterKind,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionLayout {
    pub headers: Vec<HeaderFooterRef>,
    pub footers: Vec<HeaderFooterRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMargins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBox {
    pub width: f32,
    pub height: f32,
    pub margins: PageMargins,
    pub content: ContentFrame,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutLine {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub style: TextStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutTableCell {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub column_span: u16,
    pub lines: Vec<LaidOutLine>,
    pub images: Vec<LaidOutTableCellImage>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutTableCellImage {
    pub resource_id: String,
    pub description: Option<String>,
    pub content_type: Option<String>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutTableRow {
    pub y: f32,
    pub height: f32,
    pub cells: Vec<LaidOutTableCell>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LaidOutBlock {
    Paragraph {
        list: Option<ListMarker>,
        lines: Vec<LaidOutLine>,
    },
    Table {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rows: Vec<LaidOutTableRow>,
    },
    Image {
        resource_id: String,
        description: Option<String>,
        content_type: Option<String>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocxPageLayout {
    pub page_index: u32,
    pub page_box: PageBox,
    pub blocks: Vec<LaidOutBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderFooterPlacement {
    pub kind: HeaderFooterKind,
    pub target: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionHeaderFooterLayout {
    pub page_box: PageBox,
    pub headers: Vec<HeaderFooterPlacement>,
    pub footers: Vec<HeaderFooterPlacement>,
}

pub fn parse_docx(archive: &OoxmlArchive) -> Result<DocxPackage, ViewerError> {
    let relationships = parse_package_relationships(archive)?;
    let main_document = relationships
        .iter()
        .find(|relationship| relationship.relationship_type == OFFICE_DOCUMENT_REL)
        .map(|relationship| {
            relationship
                .resolved_target
                .trim_start_matches('/')
                .to_string()
        })
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
                child, &styles, &numbering, package,
            )?),
            "tbl" => blocks.push(parse_table_block(child, &styles, &numbering, package)?),
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
                    let Some(num_fmt) =
                        level.child("numFmt").and_then(|node| node.attribute("val"))
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
                let Some(num_id) = child
                    .attribute("numId")
                    .and_then(|value| value.parse::<u32>().ok())
                else {
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
            default_paragraph_metrics: ParagraphMetrics {
                line_height: None,
                spacing_before: 0.0,
                spacing_after: 6.0,
            },
            run_styles: HashMap::new(),
        });
    };

    let xml = archive.read_part(styles_part)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;

    let mut default_run_style = ResolvedTextStyle::default();
    let mut default_paragraph_metrics = ParagraphMetrics {
        line_height: None,
        spacing_before: 0.0,
        spacing_after: 6.0,
    };
    let mut run_styles = HashMap::new();

    for child in &root.children {
        match child.local_name() {
            "docDefaults" => {
                if let Some(rpr_default) =
                    child.child("rPrDefault").and_then(|node| node.child("rPr"))
                {
                    default_run_style = parse_resolved_style(Some(rpr_default));
                }
                if let Some(ppr_default) =
                    child.child("pPrDefault").and_then(|node| node.child("pPr"))
                {
                    default_paragraph_metrics = parse_paragraph_metrics(Some(ppr_default));
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
                run_styles.insert(style_id.to_string(), RunStyleDefinition { based_on, style });
            }
            _ => {}
        }
    }

    Ok(StyleCatalog {
        default_run_style,
        default_paragraph_metrics,
        run_styles,
    })
}

pub fn parse_section_layouts(
    archive: &OoxmlArchive,
    package: &DocxPackage,
) -> Result<Vec<SectionLayout>, ViewerError> {
    let xml = archive.read_part(&package.main_document)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;
    let body = root.child("body").ok_or(ViewerError::InvalidDocument)?;
    let relationships = parse_document_relationships(archive, &package.main_document)?;

    let mut sections = Vec::new();
    for child in &body.children {
        if let Some(section) = section_from_element(child, &relationships)? {
            sections.push(section);
        }
    }

    if let Some(section_properties) = body.child("sectPr") {
        sections.push(parse_section_properties(
            section_properties,
            &relationships,
        )?);
    }

    Ok(sections)
}

pub fn parse_page_boxes(
    archive: &OoxmlArchive,
    package: &DocxPackage,
) -> Result<Vec<PageBox>, ViewerError> {
    let xml = archive.read_part(&package.main_document)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;
    let body = root.child("body").ok_or(ViewerError::InvalidDocument)?;

    let mut page_boxes = Vec::new();
    for child in &body.children {
        if let Some(section_properties) = section_properties_from_element(child) {
            page_boxes.push(parse_page_box(section_properties));
        }
    }

    if let Some(section_properties) = body.child("sectPr") {
        page_boxes.push(parse_page_box(section_properties));
    }

    Ok(page_boxes)
}

pub fn layout_document(
    archive: &OoxmlArchive,
    package: &DocxPackage,
) -> Result<Vec<DocxPageLayout>, ViewerError> {
    let styles = parse_style_catalog(archive, package)?;
    let blocks = parse_paragraph_blocks(archive, package)?;
    let mut page_boxes = parse_page_boxes(archive, package)?;
    if page_boxes.is_empty() {
        page_boxes.push(PageBox {
            width: 612.0,
            height: 792.0,
            margins: PageMargins {
                top: 72.0,
                right: 72.0,
                bottom: 72.0,
                left: 72.0,
            },
            content: ContentFrame {
                x: 72.0,
                y: 72.0,
                width: 468.0,
                height: 648.0,
            },
        });
    }

    let default_page_box = page_boxes[0].clone();
    let mut pages = vec![DocxPageLayout {
        page_index: 0,
        page_box: default_page_box.clone(),
        blocks: Vec::new(),
    }];
    let mut cursor_y = default_page_box.content.y;
    let content_bottom = default_page_box.content.y + default_page_box.content.height;

    for block in blocks {
        match block {
            Block::Paragraph { runs, list } => {
                let font_size = runs.first().map(|run| run.style.font_size).unwrap_or(12.0);
                let line_height = styles
                    .default_paragraph_metrics
                    .line_height
                    .unwrap_or_else(|| (font_size * 1.2).max(14.0))
                    .max(font_size);
                let paragraph_spacing_before = styles.default_paragraph_metrics.spacing_before;
                let paragraph_spacing_after = styles.default_paragraph_metrics.spacing_after;
                let full_text = runs
                    .iter()
                    .map(|run| run.text.as_str())
                    .collect::<Vec<_>>()
                    .join("");
                let segments = full_text.split('\u{000C}').collect::<Vec<_>>();

                for (segment_index, segment) in segments.iter().enumerate() {
                    if segment_index > 0 {
                        start_new_page(&mut pages, &mut cursor_y, &default_page_box);
                    }
                    cursor_y += paragraph_spacing_before;

                    let lines =
                        break_text_lines(segment, default_page_box.content.width, font_size);
                    if lines.is_empty() {
                        continue;
                    }

                    let mut current_lines = Vec::new();
                    for text in lines {
                        if cursor_y + line_height > content_bottom && !current_lines.is_empty() {
                            pages.last_mut().expect("page exists").blocks.push(
                                LaidOutBlock::Paragraph {
                                    list: list.clone(),
                                    lines: current_lines,
                                },
                            );
                            start_new_page(&mut pages, &mut cursor_y, &default_page_box);
                            current_lines = Vec::new();
                        } else if cursor_y + line_height > content_bottom {
                            start_new_page(&mut pages, &mut cursor_y, &default_page_box);
                        }

                        current_lines.push(LaidOutLine {
                            text: text.clone(),
                            x: default_page_box.content.x,
                            y: cursor_y,
                            width: estimate_text_width(&text, font_size)
                                .min(default_page_box.content.width),
                            height: line_height,
                            style: runs
                                .first()
                                .map(|run| run.style.clone())
                                .unwrap_or(TextStyle {
                                    font_family: "Times New Roman".to_string(),
                                    font_size,
                                    bold: false,
                                    italic: false,
                                    color_hex: "#000000".to_string(),
                                }),
                        });
                        cursor_y += line_height;
                    }

                    if !current_lines.is_empty() {
                        pages.last_mut().expect("page exists").blocks.push(
                            LaidOutBlock::Paragraph {
                                list: list.clone(),
                                lines: current_lines,
                            },
                        );
                    }

                    cursor_y += paragraph_spacing_after;
                }
            }
            Block::Table {
                rows,
                column_widths,
            } => {
                let mut table_start_y = cursor_y;
                let mut laid_out_rows = Vec::new();

                for row in rows {
                    let row_height = estimate_table_row_height(
                        &row,
                        default_page_box.content.width,
                        &column_widths,
                    );
                    if cursor_y + row_height > content_bottom && !laid_out_rows.is_empty() {
                        let table_height = cursor_y - table_start_y;
                        pages
                            .last_mut()
                            .expect("page exists")
                            .blocks
                            .push(LaidOutBlock::Table {
                                x: default_page_box.content.x,
                                y: table_start_y,
                                width: default_page_box.content.width,
                                height: table_height,
                                rows: laid_out_rows,
                            });
                        start_new_page(&mut pages, &mut cursor_y, &default_page_box);
                        table_start_y = cursor_y;
                        laid_out_rows = Vec::new();
                    } else if cursor_y + row_height > content_bottom {
                        start_new_page(&mut pages, &mut cursor_y, &default_page_box);
                        table_start_y = cursor_y;
                    }

                    laid_out_rows.push(layout_table_row(
                        &row,
                        default_page_box.content.x,
                        cursor_y,
                        default_page_box.content.width,
                        row_height,
                        &column_widths,
                    ));
                    cursor_y += row_height;
                }

                if !laid_out_rows.is_empty() {
                    let table_height = cursor_y - table_start_y;
                    pages
                        .last_mut()
                        .expect("page exists")
                        .blocks
                        .push(LaidOutBlock::Table {
                            x: default_page_box.content.x,
                            y: table_start_y,
                            width: default_page_box.content.width,
                            height: table_height,
                            rows: laid_out_rows,
                        });
                }

                cursor_y += 12.0;
            }
            Block::Image { image } => {
                let (image_width, image_height) =
                    image_display_size(&image, default_page_box.content.width);
                if cursor_y + image_height > content_bottom {
                    start_new_page(&mut pages, &mut cursor_y, &default_page_box);
                }
                pages
                    .last_mut()
                    .expect("page exists")
                    .blocks
                    .push(LaidOutBlock::Image {
                        resource_id: image.resource_id,
                        description: image.description,
                        content_type: image.content_type,
                        x: default_page_box.content.x,
                        y: cursor_y,
                        width: image_width,
                        height: image_height,
                    });
                cursor_y += image_height + 12.0;
            }
        }
    }

    Ok(pages)
}

pub fn layout_header_footers(
    archive: &OoxmlArchive,
    package: &DocxPackage,
) -> Result<Vec<SectionHeaderFooterLayout>, ViewerError> {
    let sections = parse_section_layouts(archive, package)?;
    let page_boxes = parse_page_boxes(archive, package)?;

    let mut layouts = Vec::new();
    for (index, section) in sections.iter().enumerate() {
        let page_box = page_boxes
            .get(index)
            .cloned()
            .or_else(|| page_boxes.first().cloned())
            .unwrap_or(PageBox {
                width: 612.0,
                height: 792.0,
                margins: PageMargins {
                    top: 72.0,
                    right: 72.0,
                    bottom: 72.0,
                    left: 72.0,
                },
                content: ContentFrame {
                    x: 72.0,
                    y: 72.0,
                    width: 468.0,
                    height: 648.0,
                },
            });

        layouts.push(SectionHeaderFooterLayout {
            headers: section
                .headers
                .iter()
                .map(|reference| HeaderFooterPlacement {
                    kind: reference.kind.clone(),
                    target: reference.target.clone(),
                    x: page_box.content.x,
                    y: ((page_box.margins.top - 24.0).max(0.0)) / 2.0,
                    width: page_box.content.width,
                    height: 24.0,
                })
                .collect(),
            footers: section
                .footers
                .iter()
                .map(|reference| HeaderFooterPlacement {
                    kind: reference.kind.clone(),
                    target: reference.target.clone(),
                    x: page_box.content.x,
                    y: page_box.height - page_box.margins.bottom
                        + ((page_box.margins.bottom - 24.0).max(0.0) / 2.0),
                    width: page_box.content.width,
                    height: 24.0,
                })
                .collect(),
            page_box,
        });
    }

    Ok(layouts)
}

pub fn build_search_pages(
    archive: &OoxmlArchive,
    package: &DocxPackage,
) -> Result<Vec<SearchPage>, ViewerError> {
    let pages = layout_document(archive, package)?;
    let mut search_pages = Vec::new();

    for page in pages {
        let mut chunks = Vec::new();
        for block in page.blocks {
            match block {
                LaidOutBlock::Paragraph { lines, .. } => {
                    let text = lines
                        .into_iter()
                        .map(|line| line.text)
                        .collect::<Vec<_>>()
                        .join(" ");
                    if !text.trim().is_empty() {
                        chunks.push(text);
                    }
                }
                LaidOutBlock::Table { rows, .. } => {
                    for row in rows {
                        let row_text = row
                            .cells
                            .into_iter()
                            .map(|cell| {
                                cell.lines
                                    .into_iter()
                                    .map(|line| line.text)
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        if !row_text.is_empty() {
                            chunks.push(row_text);
                        }
                    }
                }
                LaidOutBlock::Image { .. } => {}
            }
        }

        search_pages.push(SearchPage {
            page_index: page.page_index,
            text: chunks.join("\n"),
        });
    }

    Ok(search_pages)
}

pub fn search_document(
    archive: &OoxmlArchive,
    package: &DocxPackage,
    query: &str,
) -> Result<Vec<SearchMatch>, ViewerError> {
    let pages = build_search_pages(archive, package)?;
    Ok(search_pages(&pages, query))
}

pub fn build_selection_page_models(
    archive: &OoxmlArchive,
    package: &DocxPackage,
) -> Result<Vec<PageRenderModel>, ViewerError> {
    let pages = layout_document(archive, package)?;
    let mut models = Vec::new();

    for page in pages {
        let mut nodes = Vec::new();
        let mut anchors = Vec::new();
        let mut text_offset = 0u32;

        for block in page.blocks {
            match block {
                LaidOutBlock::Paragraph { lines, .. } => {
                    for line in lines {
                        push_text_node(
                            &mut nodes,
                            &mut anchors,
                            &mut text_offset,
                            line.text,
                            Rect {
                                x: line.x,
                                y: line.y,
                                width: line.width,
                                height: line.height,
                            },
                            line.style,
                        );
                    }
                }
                LaidOutBlock::Table {
                    x,
                    y,
                    width,
                    height,
                    rows,
                } => {
                    nodes.push(RenderNode::Box(BoxNode {
                        bounds: Rect {
                            x,
                            y,
                            width,
                            height,
                        },
                        fill_color_hex: None,
                        stroke_color_hex: Some("#CBD5E1".to_string()),
                        stroke_width: 1.0,
                    }));

                    for row in rows {
                        for cell in row.cells {
                            nodes.push(RenderNode::Box(BoxNode {
                                bounds: Rect {
                                    x: cell.x,
                                    y: cell.y,
                                    width: cell.width,
                                    height: cell.height,
                                },
                                fill_color_hex: Some("#FFFFFF".to_string()),
                                stroke_color_hex: Some("#CBD5E1".to_string()),
                                stroke_width: 1.0,
                            }));

                            for line in cell.lines {
                                if line.text.is_empty() {
                                    continue;
                                }

                                push_text_node(
                                    &mut nodes,
                                    &mut anchors,
                                    &mut text_offset,
                                    line.text,
                                    Rect {
                                        x: line.x,
                                        y: line.y,
                                        width: line.width,
                                        height: line.height,
                                    },
                                    line.style,
                                );
                            }

                            for image in cell.images {
                                let image_bytes = archive.read_part(&image.resource_id)?;
                                nodes.push(RenderNode::Image(ImageNode {
                                    resource_id: image.resource_id,
                                    description: image.description,
                                    content_type: image.content_type,
                                    data_base64: Some(
                                        base64::engine::general_purpose::STANDARD
                                            .encode(image_bytes),
                                    ),
                                    bounds: Rect {
                                        x: image.x,
                                        y: image.y,
                                        width: image.width,
                                        height: image.height,
                                    },
                                }));
                            }
                        }
                    }
                }
                LaidOutBlock::Image {
                    resource_id,
                    description,
                    content_type,
                    x,
                    y,
                    width,
                    height,
                } => {
                    let image_bytes = archive.read_part(&resource_id)?;
                    nodes.push(RenderNode::Image(ImageNode {
                        resource_id,
                        description,
                        content_type,
                        data_base64: Some(
                            base64::engine::general_purpose::STANDARD.encode(image_bytes),
                        ),
                        bounds: Rect {
                            x,
                            y,
                            width,
                            height,
                        },
                    }));
                }
            }
        }

        models.push(PageRenderModel {
            page_index: page.page_index,
            width: page.page_box.width,
            height: page.page_box.height,
            nodes,
            selection_anchors: anchors,
        });
    }

    Ok(models)
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
        let is_external = child.attribute("TargetMode") == Some("External");
        let resolved_target = if is_external {
            target.clone()
        } else {
            resolve_relationship_target(&base, &target)
                .trim_start_matches('/')
                .to_string()
        };

        if !is_external && !archive.contains_part(&resolved_target) {
            return Err(ViewerError::InvalidDocument);
        }

        relationships.push(DocxRelationship {
            id: child.required_attribute("Id")?.to_string(),
            relationship_type: child.required_attribute("Type")?.to_string(),
            target,
            resolved_target,
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

fn section_from_element(
    element: &viewer_core::xml::XmlElement,
    relationships: &[DocxRelationship],
) -> Result<Option<SectionLayout>, ViewerError> {
    let section_properties = section_properties_from_element(element);

    let Some(section_properties) = section_properties else {
        return Ok(None);
    };

    Ok(Some(parse_section_properties(
        section_properties,
        relationships,
    )?))
}

fn section_properties_from_element(
    element: &viewer_core::xml::XmlElement,
) -> Option<&viewer_core::xml::XmlElement> {
    if element.local_name() == "p" {
        element
            .child("pPr")
            .and_then(|properties| properties.child("sectPr"))
    } else {
        None
    }
}

fn parse_section_properties(
    section_properties: &viewer_core::xml::XmlElement,
    relationships: &[DocxRelationship],
) -> Result<SectionLayout, ViewerError> {
    let mut headers = Vec::new();
    let mut footers = Vec::new();

    for child in &section_properties.children {
        match child.local_name() {
            "headerReference" => headers.push(resolve_header_footer_reference(
                child,
                relationships,
                HEADER_REL,
            )?),
            "footerReference" => footers.push(resolve_header_footer_reference(
                child,
                relationships,
                FOOTER_REL,
            )?),
            _ => {}
        }
    }

    Ok(SectionLayout { headers, footers })
}

fn resolve_header_footer_reference(
    reference: &viewer_core::xml::XmlElement,
    relationships: &[DocxRelationship],
    expected_type: &str,
) -> Result<HeaderFooterRef, ViewerError> {
    let relationship_id = reference
        .attribute("id")
        .ok_or(ViewerError::InvalidDocument)?;
    let relationship = relationships
        .iter()
        .find(|relationship| {
            relationship.id == relationship_id && relationship.relationship_type == expected_type
        })
        .ok_or(ViewerError::InvalidDocument)?;

    Ok(HeaderFooterRef {
        kind: match reference.attribute("type") {
            Some("first") => HeaderFooterKind::First,
            Some("even") => HeaderFooterKind::Even,
            _ => HeaderFooterKind::Default,
        },
        target: relationship.resolved_target.clone(),
    })
}

fn parse_page_box(section_properties: &viewer_core::xml::XmlElement) -> PageBox {
    let (width, height) = section_properties
        .child("pgSz")
        .map(|node| {
            let width = node
                .attribute("w")
                .and_then(parse_twips_value)
                .unwrap_or(612.0);
            let height = node
                .attribute("h")
                .and_then(parse_twips_value)
                .unwrap_or(792.0);
            (width, height)
        })
        .unwrap_or((612.0, 792.0));

    let margins = section_properties
        .child("pgMar")
        .map(|node| PageMargins {
            top: node
                .attribute("top")
                .and_then(parse_twips_value)
                .unwrap_or(72.0),
            right: node
                .attribute("right")
                .and_then(parse_twips_value)
                .unwrap_or(72.0),
            bottom: node
                .attribute("bottom")
                .and_then(parse_twips_value)
                .unwrap_or(72.0),
            left: node
                .attribute("left")
                .and_then(parse_twips_value)
                .unwrap_or(72.0),
        })
        .unwrap_or(PageMargins {
            top: 72.0,
            right: 72.0,
            bottom: 72.0,
            left: 72.0,
        });

    PageBox {
        width,
        height,
        content: ContentFrame {
            x: margins.left,
            y: margins.top,
            width: (width - margins.left - margins.right).max(0.0),
            height: (height - margins.top - margins.bottom).max(0.0),
        },
        margins,
    }
}

fn parse_twips_value(value: &str) -> Option<f32> {
    value.parse::<f32>().ok().map(|twips| twips / 20.0)
}

fn start_new_page(pages: &mut Vec<DocxPageLayout>, cursor_y: &mut f32, page_box: &PageBox) {
    let next_index = pages.len() as u32;
    pages.push(DocxPageLayout {
        page_index: next_index,
        page_box: page_box.clone(),
        blocks: Vec::new(),
    });
    *cursor_y = page_box.content.y;
}

fn layout_table_row(
    row: &TableRow,
    x: f32,
    y: f32,
    width: f32,
    row_height: f32,
    column_widths: &[f32],
) -> LaidOutTableRow {
    let resolved_widths = resolve_column_widths(width, column_widths, row);
    let fallback_width = width / row.cells.len().max(1) as f32;
    let mut cell_x = x;
    let mut cells = Vec::new();
    let mut column_index = 0usize;

    for cell in &row.cells {
        let span = cell.column_span.max(1);
        let cell_width = resolve_spanned_width(
            &resolved_widths,
            column_index,
            span as usize,
            fallback_width * span as f32,
        );
        let (lines, images, _) = layout_table_cell_content(cell, cell_x, y, cell_width);
        cells.push(LaidOutTableCell {
            x: cell_x,
            y,
            width: cell_width,
            height: row_height,
            column_span: cell.column_span,
            lines,
            images,
        });
        cell_x += cell_width;
        column_index += span as usize;
    }

    LaidOutTableRow {
        y,
        height: row_height,
        cells,
    }
}

fn estimate_table_row_height(row: &TableRow, width: f32, column_widths: &[f32]) -> f32 {
    let resolved_widths = resolve_column_widths(width, column_widths, row);
    let fallback_width = width / row.cells.len().max(1) as f32;
    let mut max_height: f32 = 24.0;
    let mut column_index = 0usize;

    for cell in &row.cells {
        let span = cell.column_span.max(1);
        let cell_width = resolve_spanned_width(
            &resolved_widths,
            column_index,
            span as usize,
            fallback_width * span as f32,
        );
        let (_, _, content_height) = layout_table_cell_content(cell, 0.0, 0.0, cell_width);
        max_height = max_height.max(content_height);
        column_index += span as usize;
    }

    max_height
}

fn resolve_column_widths(width: f32, column_widths: &[f32], row: &TableRow) -> Vec<f32> {
    if !column_widths.is_empty() {
        let total = column_widths.iter().sum::<f32>().max(1.0);
        let scale = width / total;
        return column_widths.iter().map(|value| value * scale).collect();
    }

    let total_columns = row
        .cells
        .iter()
        .map(|cell| cell.column_span.max(1))
        .sum::<u16>()
        .max(1);
    vec![width / total_columns as f32; total_columns as usize]
}

fn resolve_spanned_width(
    column_widths: &[f32],
    column_index: usize,
    span: usize,
    fallback_width: f32,
) -> f32 {
    let resolved = column_widths
        .iter()
        .skip(column_index)
        .take(span)
        .sum::<f32>();
    if resolved > 0.0 {
        resolved
    } else {
        fallback_width
    }
}

fn layout_table_cell_content(
    cell: &TableCell,
    x: f32,
    y: f32,
    width: f32,
) -> (Vec<LaidOutLine>, Vec<LaidOutTableCellImage>, f32) {
    const CELL_PADDING_X: f32 = 6.0;
    const CELL_PADDING_Y: f32 = 6.0;
    const PARAGRAPH_GAP: f32 = 2.0;
    const IMAGE_GAP: f32 = 4.0;

    let mut lines = Vec::new();
    let mut images = Vec::new();
    let available_width = (width - CELL_PADDING_X * 2.0).max(24.0);
    let mut cursor_y = y + CELL_PADDING_Y;
    let mut has_content = false;

    for block in &cell.blocks {
        match block {
            Block::Paragraph { runs, .. } => {
                let font_size = runs.first().map(|run| run.style.font_size).unwrap_or(12.0);
                let line_height = (font_size * 1.2).max(14.0);
                let style = runs
                    .first()
                    .map(|run| run.style.clone())
                    .unwrap_or_else(default_text_style);
                let text = runs
                    .iter()
                    .map(|run| run.text.as_str())
                    .collect::<Vec<_>>()
                    .join("");
                let paragraph_lines = break_text_lines(&text, available_width, font_size);

                if has_content {
                    cursor_y += PARAGRAPH_GAP;
                }
                has_content = true;

                for line in paragraph_lines {
                    if !line.is_empty() {
                        lines.push(LaidOutLine {
                            text: line.clone(),
                            x: x + CELL_PADDING_X,
                            y: cursor_y,
                            width: estimate_text_width(&line, font_size).min(available_width),
                            height: line_height,
                            style: style.clone(),
                        });
                    }
                    cursor_y += line_height;
                }
            }
            Block::Image { image } => {
                let (image_width, image_height) = image_display_size(image, available_width);
                if has_content {
                    cursor_y += IMAGE_GAP;
                }
                has_content = true;
                images.push(LaidOutTableCellImage {
                    resource_id: image.resource_id.clone(),
                    description: image.description.clone(),
                    content_type: image.content_type.clone(),
                    x: x + CELL_PADDING_X,
                    y: cursor_y,
                    width: image_width,
                    height: image_height,
                });
                cursor_y += image_height;
            }
            Block::Table { .. } => {}
        }
    }

    let content_height = if has_content {
        (cursor_y - y) + CELL_PADDING_Y
    } else {
        24.0
    };

    (lines, images, content_height.max(24.0))
}

fn break_text_lines(text: &str, max_width: f32, font_size: f32) -> Vec<String> {
    let mut lines = Vec::new();

    for paragraph_line in text.split('\n') {
        if paragraph_line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let words = paragraph_line.split_whitespace().collect::<Vec<_>>();
        if words.is_empty() {
            lines.extend(break_token_lines(paragraph_line, max_width, font_size));
            continue;
        }

        let mut current = String::new();
        for word in words {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };

            if estimate_text_width(&candidate, font_size) <= max_width {
                current = candidate;
            } else if current.is_empty() {
                let mut broken = break_token_lines(word, max_width, font_size);
                if let Some(last) = broken.pop() {
                    lines.extend(broken);
                    current = last;
                }
            } else {
                lines.push(current);
                if estimate_text_width(word, font_size) <= max_width {
                    current = word.to_string();
                } else {
                    let mut broken = break_token_lines(word, max_width, font_size);
                    if let Some(last) = broken.pop() {
                        lines.extend(broken);
                        current = last;
                    } else {
                        current = String::new();
                    }
                }
            }
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }

    lines
}

fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    text.chars()
        .map(|character| estimated_char_width(character, font_size))
        .sum()
}

fn parse_paragraph_runs(
    paragraph: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
) -> Vec<TextRun> {
    let mut runs = Vec::new();
    collect_paragraph_runs(paragraph, styles, &mut runs);

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
    } else if images.is_empty() {
        blocks.push(Block::Paragraph {
            runs: vec![TextRun {
                text: String::new(),
                style: materialize_text_style(&styles.default_run_style),
            }],
            list,
        });
    }
    blocks.extend(images.into_iter().map(|image| Block::Image { image }));

    Ok(blocks)
}

fn parse_table_block(
    table: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
    numbering: &NumberingCatalog,
    package: &DocxPackage,
) -> Result<Block, ViewerError> {
    let column_widths = table
        .child("tblGrid")
        .map(|grid| {
            grid.children
                .iter()
                .filter(|child| child.local_name() == "gridCol")
                .filter_map(|child| child.attribute("w").and_then(parse_twips_value))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
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

            cells.push(parse_table_cell(cell, styles, numbering, package)?);
        }

        rows.push(TableRow { cells });
    }

    Ok(Block::Table {
        rows,
        column_widths,
    })
}

fn parse_table_cell(
    cell: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
    numbering: &NumberingCatalog,
    package: &DocxPackage,
) -> Result<TableCell, ViewerError> {
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
    collect_table_cell_blocks(cell, styles, numbering, package, &mut blocks)?;

    Ok(TableCell {
        blocks,
        column_span,
        row_merge,
    })
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
            "br" => {
                if child.attribute("type") == Some("page") {
                    text.push('\u{000C}');
                } else {
                    text.push('\n');
                }
            }
            // Word persists rendered pagination hints with this marker.
            // Treat it as a hard page boundary to keep saved pagination closer to Word.
            "lastRenderedPageBreak" => text.push('\u{000C}'),
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
    let extent = find_descendant(drawing, "extent");
    let display_width = extent
        .and_then(|node| node.attribute("cx"))
        .and_then(parse_emu_value);
    let display_height = extent
        .and_then(|node| node.attribute("cy"))
        .and_then(parse_emu_value);

    Ok(ImageReference {
        resource_id: media.resolved_target.clone(),
        description,
        content_type: infer_content_type(&media.resolved_target),
        display_width,
        display_height,
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

fn parse_paragraph_metrics(
    paragraph_properties: Option<&viewer_core::xml::XmlElement>,
) -> ParagraphMetrics {
    let spacing = paragraph_properties.and_then(|node| node.child("spacing"));
    ParagraphMetrics {
        line_height: spacing.and_then(|node| node.attribute("line").and_then(parse_twips_value)),
        spacing_before: spacing
            .and_then(|node| node.attribute("before").and_then(parse_twips_value))
            .unwrap_or(0.0),
        spacing_after: spacing
            .and_then(|node| node.attribute("after").and_then(parse_twips_value))
            .unwrap_or(6.0),
    }
}

fn parse_emu_value(value: &str) -> Option<f32> {
    value.parse::<f32>().ok().map(|emu| emu / 12_700.0)
}

fn image_display_size(image: &ImageReference, max_width: f32) -> (f32, f32) {
    match (image.display_width, image.display_height) {
        (Some(width), Some(height)) if width > 0.0 && height > 0.0 => {
            if width <= max_width {
                (width, height)
            } else {
                let scale = max_width / width;
                (max_width, height * scale)
            }
        }
        _ => {
            let width = max_width.min(192.0);
            (width, (width * 0.75).max(96.0))
        }
    }
}

fn break_token_lines(token: &str, max_width: f32, font_size: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0.0;

    for character in token.chars() {
        let char_width = estimated_char_width(character, font_size);
        if !current.is_empty() && current_width + char_width > max_width {
            lines.push(current);
            current = String::new();
            current_width = 0.0;
        }
        current.push(character);
        current_width += char_width;
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

fn estimated_char_width(character: char, font_size: f32) -> f32 {
    if character.is_whitespace() {
        return font_size * 0.35;
    }
    if is_wide_character(character) {
        return font_size;
    }
    if character.is_ascii_punctuation() {
        return font_size * 0.45;
    }
    font_size * 0.55
}

fn is_wide_character(character: char) -> bool {
    matches!(
        character as u32,
        0x1100..=0x11FF
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE6F
            | 0xFF01..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F300..=0x1FAFF
    )
}

fn collect_paragraph_runs(
    element: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
    runs: &mut Vec<TextRun>,
) {
    for child in &element.children {
        if child.local_name() == "r" {
            let run = parse_run(child, styles);
            if !run.text.is_empty() {
                runs.push(run);
            }
            continue;
        }

        collect_paragraph_runs(child, styles, runs);
    }
}

fn collect_table_cell_blocks(
    element: &viewer_core::xml::XmlElement,
    styles: &StyleCatalog,
    numbering: &NumberingCatalog,
    package: &DocxPackage,
    blocks: &mut Vec<Block>,
) -> Result<(), ViewerError> {
    for child in &element.children {
        if child.local_name() == "p" {
            blocks.extend(parse_paragraph_blocks_with_media(
                child,
                styles,
                numbering,
                Some(package),
            )?);
            continue;
        }

        collect_table_cell_blocks(child, styles, numbering, package, blocks)?;
    }

    Ok(())
}

fn push_text_node(
    nodes: &mut Vec<RenderNode>,
    anchors: &mut Vec<SelectionAnchor>,
    text_offset: &mut u32,
    text: String,
    bounds: Rect,
    style: TextStyle,
) {
    if text.is_empty() {
        return;
    }

    let node_index = nodes.len() as u32;
    let start = *text_offset;
    let end = start + text.chars().count() as u32;
    let char_width = bounds.width / text.chars().count().max(1) as f32;

    nodes.push(RenderNode::Text(TextNode {
        text: text.clone(),
        bounds: bounds.clone(),
        style,
        range: TextRange { start, end },
    }));

    for (char_index, _) in text.chars().enumerate() {
        anchors.push(SelectionAnchor {
            node_index,
            char_index: char_index as u32,
            x: bounds.x + (char_width * char_index as f32),
            y: bounds.y,
        });
    }
    anchors.push(SelectionAnchor {
        node_index,
        char_index: text.chars().count() as u32,
        x: bounds.x + bounds.width,
        y: bounds.y,
    });

    *text_offset = end;
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

fn parse_resolved_style(
    run_properties: Option<&viewer_core::xml::XmlElement>,
) -> ResolvedTextStyle {
    let Some(run_properties) = run_properties else {
        return ResolvedTextStyle::default();
    };

    let font_family = run_properties
        .child("rFonts")
        .and_then(|fonts| {
            fonts
                .attribute("ascii")
                .or_else(|| fonts.attribute("hAnsi"))
        })
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
        font_family: style.font_family.clone().unwrap_or(fallback.font_family),
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

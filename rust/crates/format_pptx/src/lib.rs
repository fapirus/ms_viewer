use base64::Engine;
use format_shared::{parse_shared_package, resolve_relationship_target};
use std::collections::HashMap;
use viewer_core::archive::OoxmlArchive;
use viewer_core::model::{
    BoxNode, ImageNode, ImageReference, PageRenderModel, ParagraphAlignment, Rect, RenderNode,
    SelectionAnchor, TextNode, TextRange, TextStyle,
};
use viewer_core::search::{search_pages, SearchMatch, SearchPage};
use viewer_core::xml::{parse_document, XmlElement};
use viewer_core::ViewerError;

const OFFICE_DOCUMENT_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const SLIDE_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";
const SLIDE_LAYOUT_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout";
const SLIDE_MASTER_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster";
const NOTES_SLIDE_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide";
const THEME_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";
const IMAGE_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

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
    pub layout_part_name: Option<String>,
    pub notes_part_name: Option<String>,
    pub has_transition: bool,
    pub ignored_animation_nodes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideLayoutReference {
    pub relationship_id: String,
    pub part_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideMasterReference {
    pub master_id: u32,
    pub relationship_id: String,
    pub part_name: String,
    pub theme_part_name: Option<String>,
    pub layouts: Vec<SlideLayoutReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmuRectangle {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeTransform {
    pub bounds: EmuRectangle,
    pub rotation_units: Option<i32>,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BasicShapeGeometry {
    Preset(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeFill {
    Solid(String),
    Gradient {
        start_color: String,
        end_color: String,
        angle_degrees: Option<i32>,
    },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeStroke {
    pub width_emu: Option<i64>,
    pub color: Option<String>,
    pub is_none: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicShape {
    pub shape_id: u32,
    pub name: String,
    pub geometry: BasicShapeGeometry,
    pub transform: Option<ShapeTransform>,
    pub fill: Option<ShapeFill>,
    pub stroke: Option<ShapeStroke>,
    pub has_text_body: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCrop {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlideImage {
    pub shape_id: u32,
    pub name: String,
    pub description: Option<String>,
    pub image: ImageReference,
    pub transform: Option<ShapeTransform>,
    pub crop: Option<ImageCrop>,
    pub is_external: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SlidePlaceholderKind {
    Title,
    CenteredTitle,
    Subtitle,
    Body,
    Object,
    Date,
    Footer,
    Header,
    SlideNumber,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlidePlaceholderReference {
    pub kind: SlidePlaceholderKind,
    pub index: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlideTextAlignment {
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SlideTextRunStyle {
    pub bold: bool,
    pub italic: bool,
    pub font_size_centipoints: Option<u32>,
    pub font_face: Option<String>,
    pub east_asia_font_face: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideTextRun {
    pub text: String,
    pub style: SlideTextRunStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideTextParagraph {
    pub alignment: Option<SlideTextAlignment>,
    pub level: Option<u32>,
    pub runs: Vec<SlideTextRun>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SlideTextInsets {
    pub left: i64,
    pub top: i64,
    pub right: i64,
    pub bottom: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideTextBox {
    pub shape_id: u32,
    pub name: String,
    pub bounds: Option<EmuRectangle>,
    pub insets: SlideTextInsets,
    pub placeholder: Option<SlidePlaceholderReference>,
    pub paragraphs: Vec<SlideTextParagraph>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideTableCell {
    pub column_span: u32,
    pub paragraphs: Vec<SlideTextParagraph>,
    pub fill: Option<String>,
    pub stroke: Option<ShapeStroke>,
    pub margins: SlideTextInsets,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideTableRow {
    pub height_emu: i64,
    pub cells: Vec<SlideTableCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideTable {
    pub shape_id: u32,
    pub name: String,
    pub bounds: EmuRectangle,
    pub column_widths_emu: Vec<i64>,
    pub rows: Vec<SlideTableRow>,
}

#[derive(Debug, Clone, PartialEq)]
struct ThemeContext {
    scheme_colors: HashMap<String, String>,
    color_mapping: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CoordinateTransform {
    tx: f64,
    ty: f64,
    sx: f64,
    sy: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlaceholderTemplate {
    bounds: Option<EmuRectangle>,
    insets: SlideTextInsets,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PptxSlideTree {
    pub presentation_part: String,
    pub presentation_size: Option<PresentationSize>,
    pub slides: Vec<SlideReference>,
    pub slide_masters: Vec<SlideMasterReference>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PartContent {
    text_boxes: Vec<SlideTextBox>,
    shapes: Vec<BasicShape>,
    images: Vec<SlideImage>,
    tables: Vec<SlideTable>,
}

impl CoordinateTransform {
    fn identity() -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            sx: 1.0,
            sy: 1.0,
        }
    }

    fn apply_rect(&self, rect: &EmuRectangle) -> EmuRectangle {
        EmuRectangle {
            x: (self.tx + self.sx * rect.x as f64).round() as i64,
            y: (self.ty + self.sy * rect.y as f64).round() as i64,
            width: (self.sx * rect.width as f64).round() as i64,
            height: (self.sy * rect.height as f64).round() as i64,
        }
    }

    fn nested_group(&self, transform: &XmlElement) -> Result<Self, ViewerError> {
        let offset = transform.child("off").ok_or(ViewerError::InvalidDocument)?;
        let extent = transform.child("ext").ok_or(ViewerError::InvalidDocument)?;
        let child_offset = transform
            .child("chOff")
            .ok_or(ViewerError::InvalidDocument)?;
        let child_extent = transform
            .child("chExt")
            .ok_or(ViewerError::InvalidDocument)?;

        let off_x = parse_i64_attribute(offset, "x")? as f64;
        let off_y = parse_i64_attribute(offset, "y")? as f64;
        let ext_x = parse_i64_attribute(extent, "cx")? as f64;
        let ext_y = parse_i64_attribute(extent, "cy")? as f64;
        let ch_off_x = parse_i64_attribute(child_offset, "x")? as f64;
        let ch_off_y = parse_i64_attribute(child_offset, "y")? as f64;
        let ch_ext_x = parse_i64_attribute(child_extent, "cx")? as f64;
        let ch_ext_y = parse_i64_attribute(child_extent, "cy")? as f64;

        let scale_x = if ch_ext_x.abs() < f64::EPSILON {
            1.0
        } else {
            ext_x / ch_ext_x
        };
        let scale_y = if ch_ext_y.abs() < f64::EPSILON {
            1.0
        } else {
            ext_y / ch_ext_y
        };

        Ok(Self {
            tx: self.tx + self.sx * (off_x - (ch_off_x * scale_x)),
            ty: self.ty + self.sy * (off_y - (ch_off_y * scale_y)),
            sx: self.sx * scale_x,
            sy: self.sy * scale_y,
        })
    }
}

pub fn parse_pptx(archive: &OoxmlArchive) -> Result<PptxSlideTree, ViewerError> {
    parse_slide_tree(archive)
}

pub fn parse_slide_text_boxes(
    archive: &OoxmlArchive,
    slide_part_name: &str,
) -> Result<Vec<SlideTextBox>, ViewerError> {
    Ok(collect_part_content(archive, slide_part_name, false)?.text_boxes)
}

pub fn parse_slide_basic_shapes(
    archive: &OoxmlArchive,
    slide_part_name: &str,
) -> Result<Vec<BasicShape>, ViewerError> {
    Ok(collect_part_content(archive, slide_part_name, false)?.shapes)
}

pub fn parse_slide_images(
    archive: &OoxmlArchive,
    slide_part_name: &str,
) -> Result<Vec<SlideImage>, ViewerError> {
    Ok(collect_part_content(archive, slide_part_name, false)?.images)
}

pub fn build_slide_render_model(
    archive: &OoxmlArchive,
    slide_tree: &PptxSlideTree,
    slide_index: usize,
) -> Result<PageRenderModel, ViewerError> {
    let slide = slide_tree
        .slides
        .get(slide_index)
        .ok_or(ViewerError::InvalidDocument)?;
    let slide_width = slide_tree
        .presentation_size
        .as_ref()
        .map(|size| emu_to_points(size.width_emu as i64))
        .unwrap_or(960.0);
    let slide_height = slide_tree
        .presentation_size
        .as_ref()
        .map(|size| emu_to_points(size.height_emu as i64))
        .unwrap_or(540.0);

    let theme = resolve_slide_theme_context(archive, slide_tree, slide);
    let render_content = collect_render_content(archive, slide_tree, slide)?;
    let mut nodes = Vec::new();
    let mut anchors = Vec::new();
    let mut text_offset = 0u32;

    for shape in render_content.shapes.iter().cloned() {
        let Some(transform) = shape.transform else {
            continue;
        };
        let (fill_color_hex, gradient_end_color_hex, gradient_angle_degrees) =
            normalize_render_fill(shape.fill.as_ref(), theme.as_ref());
        nodes.push(RenderNode::Box(BoxNode {
            bounds: rect_from_emu_bounds(&transform.bounds),
            fill_color_hex,
            gradient_end_color_hex,
            gradient_angle_degrees,
            stroke_color_hex: normalize_render_stroke_color(shape.stroke.as_ref(), theme.as_ref()),
            stroke_width: shape
                .stroke
                .as_ref()
                .and_then(|stroke| stroke.width_emu)
                .map(emu_to_points)
                .unwrap_or(0.0),
        }));
    }

    for image in render_content.images.iter().cloned() {
        let Some(transform) = image.transform else {
            continue;
        };

        let data_base64 = if image.is_external {
            None
        } else {
            Some(
                base64::engine::general_purpose::STANDARD
                    .encode(archive.read_part(&image.image.resource_id)?),
            )
        };

        nodes.push(RenderNode::Image(ImageNode {
            resource_id: image.image.resource_id,
            description: image.description,
            content_type: image.image.content_type,
            data_base64,
            bounds: rect_from_emu_bounds(&transform.bounds),
        }));
    }

    let render_content = collect_render_content(archive, slide_tree, slide)?;
    for table in &render_content.tables {
        build_table_nodes(table, theme.as_ref(), &mut nodes, &mut anchors, &mut text_offset);
    }

    for text_box in render_content.text_boxes.iter().cloned() {
        build_text_nodes(&text_box, &mut nodes, &mut anchors, &mut text_offset);
    }

    Ok(PageRenderModel {
        page_index: slide_index as u32,
        width: slide_width,
        height: slide_height,
        nodes,
        selection_anchors: anchors,
    })
}

pub fn build_search_pages(
    archive: &OoxmlArchive,
    slide_tree: &PptxSlideTree,
) -> Result<Vec<SearchPage>, ViewerError> {
    let mut pages = Vec::new();

    for (slide_index, slide) in slide_tree.slides.iter().enumerate() {
        let text_boxes = parse_slide_text_boxes(archive, &slide.part_name)?;
        let mut chunks = Vec::new();

        for text_box in text_boxes {
            let box_text = flatten_text_box_for_search(&text_box);
            if !box_text.trim().is_empty() {
                chunks.push(box_text);
            }
        }

        pages.push(SearchPage {
            page_index: slide_index as u32,
            text: chunks.join("\n"),
        });
    }

    Ok(pages)
}

pub fn search_slides(
    archive: &OoxmlArchive,
    slide_tree: &PptxSlideTree,
    query: &str,
) -> Result<Vec<SearchMatch>, ViewerError> {
    let pages = build_search_pages(archive, slide_tree)?;
    Ok(search_pages(&pages, query))
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
    let slides = parse_slide_references(archive, &presentation_root, &relationships)?;
    let slide_masters = parse_slide_master_references(archive, &presentation_root, &relationships)?;

    Ok(PptxSlideTree {
        presentation_part,
        presentation_size,
        slides,
        slide_masters,
    })
}

fn collect_part_content(
    archive: &OoxmlArchive,
    part_name: &str,
    allow_invalid_children: bool,
) -> Result<PartContent, ViewerError> {
    let root = read_part_root(archive, part_name)?;
    let relationships = parse_part_relationships(archive, part_name)?;
    let Some(shape_tree) = root
        .child("cSld")
        .and_then(|common_slide| common_slide.child("spTree")) else {
        return if root.local_name() == "sld" {
            Err(ViewerError::InvalidDocument)
        } else {
            Ok(PartContent::default())
        };
    };

    let mut content = PartContent::default();
    collect_shape_tree_content(
        shape_tree,
        &relationships,
        &CoordinateTransform::identity(),
        allow_invalid_children,
        &mut content,
    )?;
    Ok(content)
}

fn collect_render_content(
    archive: &OoxmlArchive,
    slide_tree: &PptxSlideTree,
    slide: &SlideReference,
) -> Result<PartContent, ViewerError> {
    let mut content = PartContent::default();
    let master = resolve_slide_master_reference(slide_tree, slide);
    let master_content = if let Some(master) = master {
        let part_content = collect_part_content(archive, &master.part_name, true)?;
        append_non_placeholder_part_content(&mut content, &part_content);
        Some(part_content)
    } else {
        None
    };

    let layout_content = if let Some(layout_part_name) = &slide.layout_part_name {
        let part_content = collect_part_content(archive, layout_part_name, true)?;
        append_non_placeholder_part_content(&mut content, &part_content);
        Some(part_content)
    } else {
        None
    };

    let mut slide_content = collect_part_content(archive, &slide.part_name, true)?;
    let placeholder_catalog = build_placeholder_catalog(layout_content.as_ref(), master_content.as_ref());
    apply_placeholder_templates(&mut slide_content.text_boxes, &placeholder_catalog);
    append_all_part_content(&mut content, slide_content);

    Ok(content)
}

fn append_non_placeholder_part_content(target: &mut PartContent, source: &PartContent) {
    target.shapes.extend(source.shapes.iter().cloned());
    target.images.extend(source.images.iter().cloned());
    target.tables.extend(source.tables.iter().cloned());
    target.text_boxes.extend(
        source
            .text_boxes
            .iter()
            .filter(|text_box| text_box.placeholder.is_none())
            .cloned(),
    );
}

fn append_all_part_content(target: &mut PartContent, source: PartContent) {
    target.shapes.extend(source.shapes);
    target.images.extend(source.images);
    target.tables.extend(source.tables);
    target.text_boxes.extend(source.text_boxes);
}

fn build_placeholder_catalog(
    layout_content: Option<&PartContent>,
    master_content: Option<&PartContent>,
) -> HashMap<SlidePlaceholderReference, PlaceholderTemplate> {
    let mut catalog = HashMap::new();

    if let Some(master_content) = master_content {
        for text_box in &master_content.text_boxes {
            let Some(placeholder) = text_box.placeholder.clone() else {
                continue;
            };

            catalog.insert(
                placeholder,
                PlaceholderTemplate {
                    bounds: text_box.bounds.clone(),
                    insets: text_box.insets.clone(),
                },
            );
        }
    }

    if let Some(layout_content) = layout_content {
        for text_box in &layout_content.text_boxes {
            let Some(placeholder) = text_box.placeholder.clone() else {
                continue;
            };

            catalog.insert(
                placeholder,
                PlaceholderTemplate {
                    bounds: text_box.bounds.clone(),
                    insets: text_box.insets.clone(),
                },
            );
        }
    }

    catalog
}

fn apply_placeholder_templates(
    text_boxes: &mut [SlideTextBox],
    placeholder_catalog: &HashMap<SlidePlaceholderReference, PlaceholderTemplate>,
) {
    for text_box in text_boxes {
        let Some(placeholder) = text_box.placeholder.as_ref() else {
            continue;
        };

        let Some(template) = placeholder_catalog
            .get(placeholder)
            .or_else(|| find_placeholder_by_kind(placeholder_catalog, placeholder))
        else {
            continue;
        };

        if text_box.bounds.is_none() {
            text_box.bounds = template.bounds.clone();
        }

        if text_box.insets == SlideTextInsets::default() {
            text_box.insets = template.insets.clone();
        }
    }
}

fn find_placeholder_by_kind<'a>(
    placeholder_catalog: &'a HashMap<SlidePlaceholderReference, PlaceholderTemplate>,
    placeholder: &SlidePlaceholderReference,
) -> Option<&'a PlaceholderTemplate> {
    placeholder_catalog
        .iter()
        .find(|(candidate, _)| candidate.kind == placeholder.kind)
        .map(|(_, template)| template)
}

fn collect_shape_tree_content(
    parent: &XmlElement,
    relationships: &[SlideRelationship],
    coordinate_transform: &CoordinateTransform,
    allow_invalid_children: bool,
    content: &mut PartContent,
) -> Result<(), ViewerError> {
    for child in &parent.children {
        match child.local_name() {
            "sp" => {
                if allow_invalid_children {
                    match parse_basic_shape_with_context(child, coordinate_transform) {
                        Ok(Some(shape)) => content.shapes.push(shape),
                        Ok(None) | Err(ViewerError::InvalidDocument) => {}
                        Err(error) => return Err(error),
                    }
                    match parse_text_box_shape_with_context(child, coordinate_transform) {
                        Ok(Some(text_box)) => content.text_boxes.push(text_box),
                        Ok(None) | Err(ViewerError::InvalidDocument) => {}
                        Err(error) => return Err(error),
                    }
                } else {
                    if let Some(shape) = parse_basic_shape_with_context(child, coordinate_transform)? {
                        content.shapes.push(shape);
                    }
                    if let Some(text_box) =
                        parse_text_box_shape_with_context(child, coordinate_transform)?
                    {
                        content.text_boxes.push(text_box);
                    }
                }
            }
            "pic" => {
                if allow_invalid_children {
                    match parse_slide_image_with_context(child, relationships, coordinate_transform)
                    {
                        Ok(image) => content.images.push(image),
                        Err(ViewerError::InvalidDocument) => {}
                        Err(error) => return Err(error),
                    }
                } else {
                    content.images.push(parse_slide_image_with_context(
                        child,
                        relationships,
                        coordinate_transform,
                    )?);
                }
            }
            "graphicFrame" => {
                if allow_invalid_children {
                    match parse_graphic_frame_table(child, coordinate_transform) {
                        Ok(Some(table)) => content.tables.push(table),
                        Ok(None) | Err(ViewerError::InvalidDocument) => {}
                        Err(error) => return Err(error),
                    }
                } else if let Some(table) = parse_graphic_frame_table(child, coordinate_transform)? {
                    content.tables.push(table);
                }
            }
            "grpSp" => {
                let nested_transform = if allow_invalid_children {
                    match child
                        .child("grpSpPr")
                        .and_then(|properties| properties.child("xfrm"))
                        .map(|transform| coordinate_transform.nested_group(transform))
                        .transpose()
                    {
                        Ok(Some(transform)) => transform,
                        Ok(None) | Err(ViewerError::InvalidDocument) => *coordinate_transform,
                        Err(error) => return Err(error),
                    }
                } else {
                    child
                        .child("grpSpPr")
                        .and_then(|properties| properties.child("xfrm"))
                        .map(|transform| coordinate_transform.nested_group(transform))
                        .transpose()?
                        .unwrap_or(*coordinate_transform)
                };
                collect_shape_tree_content(
                    child,
                    relationships,
                    &nested_transform,
                    allow_invalid_children,
                    content,
                )?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn read_part_root(archive: &OoxmlArchive, part_name: &str) -> Result<XmlElement, ViewerError> {
    let xml = archive.read_part(part_name)?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    parse_document(&text)
}

fn resolve_slide_master_reference<'a>(
    slide_tree: &'a PptxSlideTree,
    slide: &SlideReference,
) -> Option<&'a SlideMasterReference> {
    let layout_part_name = slide.layout_part_name.as_deref()?;
    slide_tree.slide_masters.iter().find(|master| {
        master
            .layouts
            .iter()
            .any(|layout| layout.part_name == layout_part_name)
    })
}

fn resolve_slide_theme_context(
    archive: &OoxmlArchive,
    slide_tree: &PptxSlideTree,
    slide: &SlideReference,
) -> Option<ThemeContext> {
    let master = resolve_slide_master_reference(slide_tree, slide)?;
    let theme_part_name = master.theme_part_name.as_deref()?;
    let theme_root = read_part_root(archive, theme_part_name).ok()?;
    let clr_scheme = theme_root.child("themeElements")?.child("clrScheme")?;
    let master_root = read_part_root(archive, &master.part_name).ok()?;
    let clr_map = master_root.child("clrMap")?;

    let mut scheme_colors = HashMap::new();
    for child in &clr_scheme.children {
        if let Some(color) = parse_theme_color(child) {
            scheme_colors.insert(child.local_name().to_string(), color);
        }
    }

    let mut color_mapping = HashMap::new();
    for key in [
        "bg1", "tx1", "bg2", "tx2", "accent1", "accent2", "accent3", "accent4", "accent5",
        "accent6", "hlink", "folHlink",
    ] {
        if let Some(value) = clr_map.attribute(key) {
            color_mapping.insert(key.to_string(), value.to_string());
        }
    }

    Some(ThemeContext {
        scheme_colors,
        color_mapping,
    })
}

fn parse_theme_color(color: &XmlElement) -> Option<String> {
    color
        .child("srgbClr")
        .and_then(|rgb| rgb.attribute("val"))
        .map(|value| format!("#{value}"))
        .or_else(|| {
            color
                .child("sysClr")
                .and_then(|system| system.attribute("lastClr"))
                .map(|value| format!("#{value}"))
        })
}

pub fn parse_part_relationships(
    archive: &OoxmlArchive,
    part_name: &str,
) -> Result<Vec<SlideRelationship>, ViewerError> {
    let relationship_part = relationship_part_name(part_name)?;
    if !archive.contains_part(&relationship_part) {
        return Ok(Vec::new());
    }
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
    archive: &OoxmlArchive,
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

        let slide_part_name = relationship.resolved_target.trim_start_matches('/').to_string();
        let slide_relationships = parse_part_relationships(archive, &slide_part_name)?;
        let slide_xml = archive.read_part(&slide_part_name)?;
        let slide_text = String::from_utf8(slide_xml).map_err(|_| ViewerError::InvalidDocument)?;
        let slide_root = parse_document(&slide_text)?;

        if slide_root.local_name() != "sld" {
            return Err(ViewerError::InvalidDocument);
        }

        slides.push(SlideReference {
            slide_id,
            relationship_id,
            part_name: slide_part_name.clone(),
            layout_part_name: slide_relationships
                .iter()
                .find(|relationship| relationship.relationship_type == SLIDE_LAYOUT_RELATIONSHIP)
                .map(|relationship| relationship.resolved_target.trim_start_matches('/').to_string()),
            notes_part_name: slide_relationships
                .iter()
                .find(|relationship| relationship.relationship_type == NOTES_SLIDE_RELATIONSHIP)
                .map(|relationship| relationship.resolved_target.trim_start_matches('/').to_string()),
            has_transition: slide_root.child("transition").is_some(),
            ignored_animation_nodes: slide_root
                .child("timing")
                .map(count_descendant_elements)
                .unwrap_or(0),
        });
    }

    Ok(slides)
}

fn parse_slide_master_references(
    archive: &OoxmlArchive,
    presentation_root: &viewer_core::xml::XmlElement,
    relationships: &[SlideRelationship],
) -> Result<Vec<SlideMasterReference>, ViewerError> {
    let Some(master_list) = presentation_root.child("sldMasterIdLst") else {
        return Ok(Vec::new());
    };

    let mut masters = Vec::new();
    for child in &master_list.children {
        if child.local_name() != "sldMasterId" {
            continue;
        }

        let relationship_id = child.required_attribute("r:id")?.to_string();
        let master_id = parse_u32_attribute(child, "id")?;
        let relationship = relationships
            .iter()
            .find(|relationship| {
                relationship.id == relationship_id
                    && relationship.relationship_type == SLIDE_MASTER_RELATIONSHIP
            })
            .ok_or(ViewerError::InvalidDocument)?;
        let master_part = relationship.resolved_target.trim_start_matches('/').to_string();
        let master_relationships = parse_part_relationships(archive, &master_part)?;

        let mut layouts = Vec::new();
        for related in &master_relationships {
            if related.relationship_type != SLIDE_LAYOUT_RELATIONSHIP {
                continue;
            }

            layouts.push(SlideLayoutReference {
                relationship_id: related.id.clone(),
                part_name: related.resolved_target.trim_start_matches('/').to_string(),
            });
        }

        let theme_part_name = master_relationships
            .iter()
            .find(|relationship| relationship.relationship_type == THEME_RELATIONSHIP)
            .map(|relationship| relationship.resolved_target.trim_start_matches('/').to_string());

        masters.push(SlideMasterReference {
            master_id,
            relationship_id,
            part_name: master_part,
            theme_part_name,
            layouts,
        });
    }

    Ok(masters)
}

fn count_descendant_elements(element: &XmlElement) -> u32 {
    element
        .children
        .iter()
        .map(|child| 1 + count_descendant_elements(child))
        .sum()
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
    element: &XmlElement,
    name: &str,
) -> Result<u32, ViewerError> {
    element
        .required_attribute(name)?
        .parse::<u32>()
        .map_err(|_| ViewerError::InvalidDocument)
}

fn parse_i64_attribute(element: &XmlElement, name: &str) -> Result<i64, ViewerError> {
    element
        .required_attribute(name)?
        .parse::<i64>()
        .map_err(|_| ViewerError::InvalidDocument)
}

fn parse_text_box_shape_with_context(
    shape: &XmlElement,
    coordinate_transform: &CoordinateTransform,
) -> Result<Option<SlideTextBox>, ViewerError> {
    let Some(non_visual) = shape.child("nvSpPr") else {
        return Ok(None);
    };
    let Some(properties) = non_visual.child("cNvPr") else {
        return Ok(None);
    };
    let Some(text_body) = shape.child("txBody") else {
        return Ok(None);
    };

    let shape_id = parse_u32_attribute(properties, "id")?;
    let name = properties.required_attribute("name")?.to_string();
    let bounds = shape
        .child("spPr")
        .and_then(|shape_properties| shape_properties.child("xfrm"))
        .map(|transform| parse_shape_transform_with_context(transform, coordinate_transform))
        .transpose()?
        .map(|transform| transform.bounds);
    let insets = text_body
        .child("bodyPr")
        .map(parse_text_insets)
        .transpose()?
        .unwrap_or_default();
    let placeholder = parse_placeholder_reference(non_visual)?;
    let paragraphs = parse_text_paragraphs(text_body)?;

    Ok(Some(SlideTextBox {
        shape_id,
        name,
        bounds,
        insets,
        placeholder,
        paragraphs,
    }))
}

fn parse_basic_shape_with_context(
    shape: &XmlElement,
    coordinate_transform: &CoordinateTransform,
) -> Result<Option<BasicShape>, ViewerError> {
    let Some(non_visual) = shape.child("nvSpPr") else {
        return Ok(None);
    };
    let Some(properties) = non_visual.child("cNvPr") else {
        return Ok(None);
    };
    let Some(shape_properties) = shape.child("spPr") else {
        return Ok(None);
    };
    let Some(geometry) = parse_basic_shape_geometry(shape_properties)? else {
        return Ok(None);
    };

    let shape_id = parse_u32_attribute(properties, "id")?;
    let name = properties.required_attribute("name")?.to_string();
    let transform = shape_properties
        .child("xfrm")
        .map(|transform| parse_shape_transform_with_context(transform, coordinate_transform))
        .transpose()?;
    let fill = parse_shape_fill(shape_properties)?;
    let stroke = parse_shape_stroke(shape_properties)?;

    Ok(Some(BasicShape {
        shape_id,
        name,
        geometry,
        transform,
        fill,
        stroke,
        has_text_body: shape.child("txBody").is_some(),
    }))
}

fn parse_slide_image_with_context(
    picture: &XmlElement,
    relationships: &[SlideRelationship],
    coordinate_transform: &CoordinateTransform,
) -> Result<SlideImage, ViewerError> {
    let non_visual = picture.child("nvPicPr").ok_or(ViewerError::InvalidDocument)?;
    let properties = non_visual.child("cNvPr").ok_or(ViewerError::InvalidDocument)?;
    let blip_fill = picture.child("blipFill").ok_or(ViewerError::InvalidDocument)?;
    let blip = blip_fill.child("blip").ok_or(ViewerError::InvalidDocument)?;
    let reference_id = blip
        .attribute("r:embed")
        .or_else(|| blip.attribute("r:link"))
        .ok_or(ViewerError::InvalidDocument)?
        .to_string();
    let relationship = relationships
        .iter()
        .find(|relationship| {
            relationship.id == reference_id && relationship.relationship_type == IMAGE_RELATIONSHIP
        })
        .ok_or(ViewerError::InvalidDocument)?;
    let is_external = relationship.target.starts_with("http://")
        || relationship.target.starts_with("https://")
        || relationship.resolved_target.starts_with("http://")
        || relationship.resolved_target.starts_with("https://");
    let resource_id = relationship.resolved_target.trim_start_matches('/').to_string();
    let name = properties.required_attribute("name")?.to_string();
    let description = properties
        .attribute("descr")
        .map(ToOwned::to_owned)
        .or_else(|| Some(name.clone()));
    let transform = picture
        .child("spPr")
        .and_then(|shape_properties| shape_properties.child("xfrm"))
        .map(|transform| parse_shape_transform_with_context(transform, coordinate_transform))
        .transpose()?;
    let crop = blip_fill.child("srcRect").map(parse_image_crop).transpose()?;

    Ok(SlideImage {
        shape_id: parse_u32_attribute(properties, "id")?,
        name,
        description: description.clone(),
        image: ImageReference {
            resource_id: resource_id.clone(),
            description,
            content_type: infer_image_content_type(&resource_id),
            display_width: transform.as_ref().map(|transform| transform.bounds.width as f32),
            display_height: transform.as_ref().map(|transform| transform.bounds.height as f32),
        },
        transform,
        crop,
        is_external,
    })
}

fn parse_shape_transform_with_context(
    transform: &XmlElement,
    coordinate_transform: &CoordinateTransform,
) -> Result<ShapeTransform, ViewerError> {
    let offset = transform.child("off").ok_or(ViewerError::InvalidDocument)?;
    let extent = transform.child("ext").ok_or(ViewerError::InvalidDocument)?;
    let raw_bounds = EmuRectangle {
        x: parse_i64_attribute(offset, "x")?,
        y: parse_i64_attribute(offset, "y")?,
        width: parse_i64_attribute(extent, "cx")?,
        height: parse_i64_attribute(extent, "cy")?,
    };

    Ok(ShapeTransform {
        bounds: coordinate_transform.apply_rect(&raw_bounds),
        rotation_units: transform
            .attribute("rot")
            .map(|rotation| rotation.parse::<i32>().map_err(|_| ViewerError::InvalidDocument))
            .transpose()?,
        flip_horizontal: transform.attribute("flipH") == Some("1"),
        flip_vertical: transform.attribute("flipV") == Some("1"),
    })
}

fn parse_basic_shape_geometry(
    shape_properties: &XmlElement,
) -> Result<Option<BasicShapeGeometry>, ViewerError> {
    let Some(geometry) = shape_properties.child("prstGeom") else {
        return Ok(None);
    };

    Ok(Some(BasicShapeGeometry::Preset(
        geometry.required_attribute("prst")?.to_string(),
    )))
}

fn parse_placeholder_reference(
    non_visual: &XmlElement,
) -> Result<Option<SlidePlaceholderReference>, ViewerError> {
    let Some(placeholder) = non_visual.child("nvPr").and_then(|properties| properties.child("ph")) else {
        return Ok(None);
    };

    let placeholder_type = placeholder.attribute("type").unwrap_or("body");
    let kind = match placeholder_type {
        "title" => SlidePlaceholderKind::Title,
        "ctrTitle" => SlidePlaceholderKind::CenteredTitle,
        "subTitle" => SlidePlaceholderKind::Subtitle,
        "body" => SlidePlaceholderKind::Body,
        "obj" => SlidePlaceholderKind::Object,
        "dt" => SlidePlaceholderKind::Date,
        "ftr" => SlidePlaceholderKind::Footer,
        "hdr" => SlidePlaceholderKind::Header,
        "sldNum" => SlidePlaceholderKind::SlideNumber,
        other => SlidePlaceholderKind::Other(other.to_string()),
    };
    let index = placeholder
        .attribute("idx")
        .map(|index| index.parse::<u32>().map_err(|_| ViewerError::InvalidDocument))
        .transpose()?;

    Ok(Some(SlidePlaceholderReference { kind, index }))
}

fn parse_shape_fill(shape_properties: &XmlElement) -> Result<Option<ShapeFill>, ViewerError> {
    if shape_properties.child("noFill").is_some() {
        return Ok(Some(ShapeFill::None));
    }

    if let Some(gradient_fill) = shape_properties.child("gradFill") {
        return Ok(Some(parse_gradient_fill(gradient_fill)?));
    }

    let Some(solid_fill) = shape_properties.child("solidFill") else {
        return Ok(None);
    };

    Ok(Some(ShapeFill::Solid(parse_color_value(solid_fill)?)))
}

fn parse_shape_stroke(shape_properties: &XmlElement) -> Result<Option<ShapeStroke>, ViewerError> {
    let Some(line) = shape_properties.child("ln") else {
        return Ok(None);
    };

    parse_line_stroke(line)
}

fn parse_line_stroke(line: &XmlElement) -> Result<Option<ShapeStroke>, ViewerError> {
    let width_emu = line
        .attribute("w")
        .map(|width| width.parse::<i64>().map_err(|_| ViewerError::InvalidDocument))
        .transpose()?;

    if line.child("noFill").is_some() {
        return Ok(Some(ShapeStroke {
            width_emu,
            color: None,
            is_none: true,
        }));
    }

    let color = line
        .child("solidFill")
        .map(parse_color_value)
        .transpose()?;

    Ok(Some(ShapeStroke {
        width_emu,
        color,
        is_none: false,
    }))
}

fn parse_gradient_fill(gradient_fill: &XmlElement) -> Result<ShapeFill, ViewerError> {
    let gradient_stops = gradient_fill
        .child("gsLst")
        .ok_or(ViewerError::InvalidDocument)?
        .children
        .iter()
        .filter(|child| child.local_name() == "gs")
        .collect::<Vec<_>>();

    let first = gradient_stops.first().ok_or(ViewerError::InvalidDocument)?;
    let last = gradient_stops.last().ok_or(ViewerError::InvalidDocument)?;
    let angle_degrees = gradient_fill
        .child("lin")
        .and_then(|line| line.attribute("ang"))
        .map(|angle| angle.parse::<i32>().map(|value| value / 60_000).map_err(|_| ViewerError::InvalidDocument))
        .transpose()?;

    Ok(ShapeFill::Gradient {
        start_color: parse_color_value(first)?,
        end_color: parse_color_value(last)?,
        angle_degrees,
    })
}

fn parse_text_paragraphs(text_body: &XmlElement) -> Result<Vec<SlideTextParagraph>, ViewerError> {
    let mut paragraphs = Vec::new();
    for child in &text_body.children {
        if child.local_name() != "p" {
            continue;
        }

        paragraphs.push(parse_text_paragraph(child)?);
    }

    Ok(paragraphs)
}

fn parse_text_paragraph(paragraph: &XmlElement) -> Result<SlideTextParagraph, ViewerError> {
    let alignment = paragraph
        .child("pPr")
        .and_then(|properties| properties.attribute("algn"))
        .and_then(parse_alignment);
    let level = paragraph
        .child("pPr")
        .and_then(|properties| properties.attribute("lvl"))
        .map(|level| level.parse::<u32>().map_err(|_| ViewerError::InvalidDocument))
        .transpose()?;

    let mut runs = Vec::new();
    for child in &paragraph.children {
        match child.local_name() {
            "r" => runs.push(parse_text_run(child)?),
            "br" => runs.push(SlideTextRun {
                text: "\n".to_string(),
                style: SlideTextRunStyle::default(),
            }),
            "fld" => {
                if let Some(text) = field_text(child) {
                    runs.push(SlideTextRun {
                        text,
                        style: child
                            .child("rPr")
                            .map(parse_text_run_style)
                            .transpose()?
                            .unwrap_or_default(),
                    });
                }
            }
            _ => {}
        }
    }

    Ok(SlideTextParagraph {
        alignment,
        level,
        runs,
    })
}

fn parse_text_run(run: &XmlElement) -> Result<SlideTextRun, ViewerError> {
    let text = run
        .child("t")
        .map(|text| text.text.clone())
        .unwrap_or_default();
    let style = run
        .child("rPr")
        .map(parse_text_run_style)
        .transpose()?
        .unwrap_or_default();

    Ok(SlideTextRun { text, style })
}

fn parse_text_run_style(run_properties: &XmlElement) -> Result<SlideTextRunStyle, ViewerError> {
    let font_size_centipoints = run_properties
        .attribute("sz")
        .map(|size| size.parse::<u32>().map_err(|_| ViewerError::InvalidDocument))
        .transpose()?;
    let bold = run_properties.attribute("b") == Some("1");
    let italic = run_properties.attribute("i") == Some("1");
    let font_face = run_properties
        .child("latin")
        .and_then(|latin| latin.attribute("typeface"))
        .map(ToOwned::to_owned);
    let east_asia_font_face = run_properties
        .child("ea")
        .and_then(|east_asia| east_asia.attribute("typeface"))
        .map(ToOwned::to_owned);
    let color = run_properties
        .child("solidFill")
        .and_then(|fill| fill.child("srgbClr"))
        .and_then(|color| color.attribute("val"))
        .map(|value| format!("#{value}"));

    Ok(SlideTextRunStyle {
        bold,
        italic,
        font_size_centipoints,
        font_face,
        east_asia_font_face,
        color,
    })
}

fn parse_color_value(fill: &XmlElement) -> Result<String, ViewerError> {
    if let Some(rgb) = fill.child("srgbClr").and_then(|color| color.attribute("val")) {
        return Ok(format!("#{rgb}"));
    }

    if let Some(system) = fill.child("sysClr").and_then(|color| color.attribute("lastClr")) {
        return Ok(format!("#{system}"));
    }

    if let Some(scheme) = fill.child("schemeClr").and_then(|color| color.attribute("val")) {
        return Ok(format!("scheme:{scheme}"));
    }

    if let Some(preset) = fill.child("prstClr").and_then(|color| color.attribute("val")) {
        return Ok(format!("preset:{preset}"));
    }

    Err(ViewerError::InvalidDocument)
}

fn parse_image_crop(src_rect: &XmlElement) -> Result<ImageCrop, ViewerError> {
    Ok(ImageCrop {
        left: parse_u32_optional_attribute(src_rect, "l")?.unwrap_or(0),
        top: parse_u32_optional_attribute(src_rect, "t")?.unwrap_or(0),
        right: parse_u32_optional_attribute(src_rect, "r")?.unwrap_or(0),
        bottom: parse_u32_optional_attribute(src_rect, "b")?.unwrap_or(0),
    })
}

fn parse_text_insets(body_properties: &XmlElement) -> Result<SlideTextInsets, ViewerError> {
    Ok(SlideTextInsets {
        left: parse_i64_optional_attribute(body_properties, "lIns")?.unwrap_or(0),
        top: parse_i64_optional_attribute(body_properties, "tIns")?.unwrap_or(0),
        right: parse_i64_optional_attribute(body_properties, "rIns")?.unwrap_or(0),
        bottom: parse_i64_optional_attribute(body_properties, "bIns")?.unwrap_or(0),
    })
}

fn parse_graphic_frame_table(
    graphic_frame: &XmlElement,
    coordinate_transform: &CoordinateTransform,
) -> Result<Option<SlideTable>, ViewerError> {
    let Some(non_visual) = graphic_frame.child("nvGraphicFramePr") else {
        return Ok(None);
    };
    let Some(properties) = non_visual.child("cNvPr") else {
        return Ok(None);
    };
    let Some(graphic) = graphic_frame.child("graphic") else {
        return Ok(None);
    };
    let Some(graphic_data) = graphic.child("graphicData") else {
        return Ok(None);
    };
    let Some(table) = graphic_data.child("tbl") else {
        return Ok(None);
    };

    let transform = graphic_frame
        .child("xfrm")
        .map(|transform| parse_shape_transform_with_context(transform, coordinate_transform))
        .transpose()?
        .ok_or(ViewerError::InvalidDocument)?;

    let column_widths_emu = table
        .child("tblGrid")
        .map(|grid| {
            grid.children
                .iter()
                .filter(|child| child.local_name() == "gridCol")
                .map(|column| parse_i64_attribute(column, "w"))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    let mut rows = Vec::new();
    for row in &table.children {
        if row.local_name() != "tr" {
            continue;
        }

        let height_emu = parse_i64_attribute(row, "h").unwrap_or(0);
        let mut cells = Vec::new();
        for cell in &row.children {
            if cell.local_name() != "tc" {
                continue;
            }

            let properties = cell.child("tcPr");
            let column_span = properties
                .and_then(|properties| properties.attribute("gridSpan"))
                .map(|span| span.parse::<u32>().map_err(|_| ViewerError::InvalidDocument))
                .transpose()?
                .unwrap_or(1);
            let fill = properties
                .and_then(|properties| properties.child("solidFill"))
                .map(parse_color_value)
                .transpose()?;
            let stroke = properties
                .and_then(|properties| {
                    ["lnL", "lnR", "lnT", "lnB"]
                        .iter()
                        .find_map(|name| properties.child(name))
                })
                .map(parse_line_stroke)
                .transpose()?
                .flatten();
            let margins = properties
                .map(parse_table_cell_margins)
                .transpose()?
                .unwrap_or_default();
            let paragraphs = cell
                .child("txBody")
                .map(parse_text_paragraphs)
                .transpose()?
                .unwrap_or_default();

            cells.push(SlideTableCell {
                column_span,
                paragraphs,
                fill,
                stroke,
                margins,
            });
        }

        rows.push(SlideTableRow { height_emu, cells });
    }

    Ok(Some(SlideTable {
        shape_id: parse_u32_attribute(properties, "id")?,
        name: properties.required_attribute("name")?.to_string(),
        bounds: transform.bounds,
        column_widths_emu,
        rows,
    }))
}

fn parse_table_cell_margins(properties: &XmlElement) -> Result<SlideTextInsets, ViewerError> {
    Ok(SlideTextInsets {
        left: parse_i64_optional_attribute(properties, "marL")?.unwrap_or(0),
        top: parse_i64_optional_attribute(properties, "marT")?.unwrap_or(0),
        right: parse_i64_optional_attribute(properties, "marR")?.unwrap_or(0),
        bottom: parse_i64_optional_attribute(properties, "marB")?.unwrap_or(0),
    })
}

fn parse_u32_optional_attribute(
    element: &XmlElement,
    name: &str,
) -> Result<Option<u32>, ViewerError> {
    element
        .attribute(name)
        .map(|value| value.parse::<u32>().map_err(|_| ViewerError::InvalidDocument))
        .transpose()
}

fn parse_i64_optional_attribute(
    element: &XmlElement,
    name: &str,
) -> Result<Option<i64>, ViewerError> {
    element
        .attribute(name)
        .map(|value| value.parse::<i64>().map_err(|_| ViewerError::InvalidDocument))
        .transpose()
}

fn infer_image_content_type(path: &str) -> Option<String> {
    let normalized = path.rsplit_once('.').map(|(_, ext)| ext.to_ascii_lowercase())?;
    let content_type = match normalized.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => return None,
    };

    Some(content_type.to_string())
}

fn parse_alignment(value: &str) -> Option<SlideTextAlignment> {
    match value {
        "l" => Some(SlideTextAlignment::Left),
        "ctr" => Some(SlideTextAlignment::Center),
        "r" => Some(SlideTextAlignment::Right),
        "just" => Some(SlideTextAlignment::Justified),
        _ => None,
    }
}

fn field_text(field: &XmlElement) -> Option<String> {
    field.child("t").map(|text| text.text.clone())
}

fn flatten_text_box_for_search(text_box: &SlideTextBox) -> String {
    text_box
        .paragraphs
        .iter()
        .map(flatten_paragraph_for_search)
        .filter(|paragraph| !paragraph.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn flatten_paragraph_for_search(paragraph: &SlideTextParagraph) -> String {
    paragraph
        .runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<String>()
        .replace('\u{000B}', "\n")
}

fn rect_from_emu_bounds(bounds: &EmuRectangle) -> Rect {
    Rect {
        x: emu_to_points(bounds.x),
        y: emu_to_points(bounds.y),
        width: emu_to_points(bounds.width),
        height: emu_to_points(bounds.height),
    }
}

fn emu_to_points(value: i64) -> f32 {
    value as f32 / 12_700.0
}

fn normalize_render_fill(
    fill: Option<&ShapeFill>,
    theme_context: Option<&ThemeContext>,
) -> (Option<String>, Option<String>, Option<f32>) {
    match fill {
        Some(ShapeFill::Solid(color)) => (
            resolve_render_color(color, theme_context),
            None,
            None,
        ),
        Some(ShapeFill::Gradient {
            start_color,
            end_color,
            angle_degrees,
        }) => (
            resolve_render_color(start_color, theme_context),
            resolve_render_color(end_color, theme_context),
            angle_degrees.map(|angle| angle as f32),
        ),
        _ => (None, None, None),
    }
}

fn normalize_render_stroke_color(
    stroke: Option<&ShapeStroke>,
    theme_context: Option<&ThemeContext>,
) -> Option<String> {
    let Some(stroke) = stroke else {
        return None;
    };
    if stroke.is_none {
        return None;
    }
    stroke.color
        .as_ref()
        .and_then(|color| resolve_render_color(color, theme_context))
}

fn resolve_render_color(
    color: &str,
    theme_context: Option<&ThemeContext>,
) -> Option<String> {
    if color.starts_with('#') {
        return Some(color.to_string());
    }

    let Some(theme_context) = theme_context else {
        return None;
    };
    let Some(scheme) = color.strip_prefix("scheme:") else {
        return None;
    };

    let mapped_scheme = theme_context
        .color_mapping
        .get(scheme)
        .map(String::as_str)
        .unwrap_or(scheme);
    theme_context.scheme_colors.get(mapped_scheme).cloned()
}

fn build_table_nodes(
    table: &SlideTable,
    theme_context: Option<&ThemeContext>,
    nodes: &mut Vec<RenderNode>,
    anchors: &mut Vec<SelectionAnchor>,
    text_offset: &mut u32,
) {
    let total_width = table
        .column_widths_emu
        .iter()
        .copied()
        .sum::<i64>()
        .max(1);
    let total_height = table.rows.iter().map(|row| row.height_emu).sum::<i64>().max(1);
    let mut cursor_y = table.bounds.y;

    for row in &table.rows {
        let row_height = if row.height_emu > 0 {
            row.height_emu
        } else {
            ((table.bounds.height as f64 / table.rows.len().max(1) as f64).round() as i64).max(1)
        };
        let mut cursor_x = table.bounds.x;
        let mut column_index = 0usize;

        for cell in &row.cells {
            let span = cell.column_span.max(1) as usize;
            let column_width = table
                .column_widths_emu
                .iter()
                .skip(column_index)
                .take(span)
                .copied()
                .sum::<i64>()
                .max(1);
            let width_ratio = column_width as f64 / total_width as f64;
            let height_ratio = row_height as f64 / total_height as f64;
            let cell_width = (table.bounds.width as f64 * width_ratio).round() as i64;
            let cell_height = (table.bounds.height as f64 * height_ratio).round() as i64;
            let cell_bounds = EmuRectangle {
                x: cursor_x,
                y: cursor_y,
                width: cell_width.max(1),
                height: cell_height.max(1),
            };

            let fill_color = cell
                .fill
                .as_deref()
                .and_then(|fill| resolve_render_color(fill, theme_context));
            let stroke_color = normalize_render_stroke_color(cell.stroke.as_ref(), theme_context);
            let stroke_width = cell
                .stroke
                .as_ref()
                .and_then(|stroke| stroke.width_emu)
                .map(emu_to_points)
                .unwrap_or(0.5)
                .max(0.5);

            nodes.push(RenderNode::Box(BoxNode {
                bounds: rect_from_emu_bounds(&cell_bounds),
                fill_color_hex: fill_color,
                gradient_end_color_hex: None,
                gradient_angle_degrees: None,
                stroke_color_hex: stroke_color,
                stroke_width,
            }));

            build_text_nodes(
                &SlideTextBox {
                    shape_id: table.shape_id,
                    name: table.name.clone(),
                    bounds: Some(cell_bounds),
                    insets: cell.margins.clone(),
                    placeholder: None,
                    paragraphs: cell.paragraphs.clone(),
                },
                nodes,
                anchors,
                text_offset,
            );

            cursor_x += cell_width;
            column_index += span;
        }

        cursor_y += row_height;
    }
}

fn build_text_nodes(
    text_box: &SlideTextBox,
    nodes: &mut Vec<RenderNode>,
    anchors: &mut Vec<SelectionAnchor>,
    text_offset: &mut u32,
) {
    let Some(bounds) = &text_box.bounds else {
        return;
    };

    let left_inset = emu_to_points(text_box.insets.left);
    let top_inset = emu_to_points(text_box.insets.top);
    let right_inset = emu_to_points(text_box.insets.right);
    let bottom_inset = emu_to_points(text_box.insets.bottom);
    let content_width = (emu_to_points(bounds.width) - left_inset - right_inset).max(1.0);
    let base_x = emu_to_points(bounds.x) + left_inset;
    let mut cursor_y = emu_to_points(bounds.y) + top_inset;
    let max_bottom = emu_to_points(bounds.y + bounds.height) - bottom_inset;

    'paragraphs: for paragraph in &text_box.paragraphs {
        let alignment = paragraph
            .alignment
            .as_ref()
            .map(slide_alignment_to_paragraph_alignment)
            .unwrap_or(ParagraphAlignment::Left);
        let paragraph_font_size = paragraph_default_font_size(paragraph);
        let paragraph_spacing = (paragraph_font_size * 0.2).max(4.0);
        let indent = paragraph.level.unwrap_or(0) as f32 * 18.0;
        let line_base_x = base_x + indent;
        let line_available_width = (content_width - indent).max(1.0);
        let lines = wrap_paragraph_lines(paragraph, line_available_width);

        for line in lines {
            if cursor_y + line.height > max_bottom {
                break 'paragraphs;
            }

            let mut cursor_x =
                resolve_line_x(line_base_x, line_available_width, line.width, &alignment);

            for span in line.spans {
                let bounds = Rect {
                    x: cursor_x,
                    y: cursor_y,
                    width: span.width.max(1.0),
                    height: line.height,
                };
                push_text_node(
                    nodes,
                    anchors,
                    text_offset,
                    span.text,
                    bounds,
                    span.style,
                );
                cursor_x += span.width;
            }

            cursor_y += line.height;
        }

        if cursor_y + paragraph_spacing > max_bottom {
            break;
        }
        cursor_y += paragraph_spacing;
    }
}

fn slide_alignment_to_paragraph_alignment(alignment: &SlideTextAlignment) -> ParagraphAlignment {
    match alignment {
        SlideTextAlignment::Left => ParagraphAlignment::Left,
        SlideTextAlignment::Center => ParagraphAlignment::Center,
        SlideTextAlignment::Right => ParagraphAlignment::Right,
        SlideTextAlignment::Justified => ParagraphAlignment::Justified,
    }
}

fn slide_run_style_to_text_style(style: &SlideTextRunStyle) -> TextStyle {
    let font_family = style
        .east_asia_font_face
        .clone()
        .or_else(|| style.font_face.clone())
        .unwrap_or_else(|| "Calibri".to_string());
    let font_size = style
        .font_size_centipoints
        .map(|size| size as f32 / 100.0)
        .unwrap_or(18.0);

    TextStyle {
        font_family,
        font_size,
        bold: style.bold,
        italic: style.italic,
        color_hex: style
            .color
            .clone()
            .unwrap_or_else(|| "#000000".to_string()),
    }
}

fn resolve_line_x(
    base_x: f32,
    available_width: f32,
    line_width: f32,
    alignment: &ParagraphAlignment,
) -> f32 {
    match alignment {
        ParagraphAlignment::Center => base_x + ((available_width - line_width).max(0.0) / 2.0),
        ParagraphAlignment::Right => base_x + (available_width - line_width).max(0.0),
        ParagraphAlignment::Left | ParagraphAlignment::Justified => base_x,
    }
}

fn estimate_text_width(text: &str, style: &TextStyle) -> f32 {
    text.chars()
        .map(|character| estimated_char_width(character, style))
        .sum::<f32>()
}

fn estimated_char_width(character: char, style: &TextStyle) -> f32 {
    let base = if is_wide_character(character) {
        style.font_size * 0.95
    } else if character.is_ascii_whitespace() {
        style.font_size * 0.33
    } else if character.is_ascii_punctuation() {
        style.font_size * 0.42
    } else {
        style.font_size * 0.56
    };

    let weight_scale = if style.bold { 1.06 } else { 1.0 };
    base * weight_scale
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

#[derive(Debug, Clone)]
struct WrappedTextSpan {
    text: String,
    style: TextStyle,
    width: f32,
}

#[derive(Debug, Clone)]
struct WrappedLine {
    spans: Vec<WrappedTextSpan>,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone)]
struct LineBuilder {
    spans: Vec<WrappedTextSpan>,
    width: f32,
    max_font_size: f32,
}

impl LineBuilder {
    fn new() -> Self {
        Self {
            spans: Vec::new(),
            width: 0.0,
            max_font_size: 0.0,
        }
    }

    fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    fn push_text(&mut self, text: String, style: TextStyle) {
        if text.is_empty() {
            return;
        }

        let width = estimate_text_width(&text, &style);
        self.max_font_size = self.max_font_size.max(style.font_size);

        if let Some(last) = self.spans.last_mut() {
            if last.style == style {
                last.text.push_str(&text);
                last.width += width;
                self.width += width;
                return;
            }
        }

        self.width += width;
        self.spans.push(WrappedTextSpan { text, style, width });
    }

    fn finish(self, fallback_font_size: f32) -> WrappedLine {
        WrappedLine {
            width: self.width,
            height: (self.max_font_size.max(fallback_font_size) * 1.2).max(18.0),
            spans: self.spans,
        }
    }
}

fn paragraph_default_font_size(paragraph: &SlideTextParagraph) -> f32 {
    paragraph
        .runs
        .iter()
        .map(|run| slide_run_style_to_text_style(&run.style).font_size)
        .find(|size| *size > 0.0)
        .unwrap_or(18.0)
}

fn wrap_paragraph_lines(paragraph: &SlideTextParagraph, max_width: f32) -> Vec<WrappedLine> {
    let default_font_size = paragraph_default_font_size(paragraph);
    let mut lines = Vec::new();
    let mut current = LineBuilder::new();

    for run in &paragraph.runs {
        let style = slide_run_style_to_text_style(&run.style);
        let parts: Vec<&str> = run.text.split('\n').collect();

        for (index, part) in parts.iter().enumerate() {
            push_wrapped_text_part(part, &style, max_width, &mut current, &mut lines, default_font_size);

            if index + 1 < parts.len() {
                flush_line(&mut current, &mut lines, default_font_size, true);
            }
        }
    }

    let should_emit_empty_line = lines.is_empty();
    flush_line(
        &mut current,
        &mut lines,
        default_font_size,
        should_emit_empty_line,
    );
    lines
}

fn push_wrapped_text_part(
    part: &str,
    style: &TextStyle,
    max_width: f32,
    current: &mut LineBuilder,
    lines: &mut Vec<WrappedLine>,
    default_font_size: f32,
) {
    for token in tokenize_text_for_layout(part) {
        let is_whitespace = token.chars().all(char::is_whitespace);
        if is_whitespace {
            if current.is_empty() {
                continue;
            }

            let width = estimate_text_width(&token, style);
            if current.width + width <= max_width {
                current.push_text(token, style.clone());
            } else {
                flush_line(current, lines, default_font_size, false);
            }
            continue;
        }

        push_non_whitespace_token(&token, style, max_width, current, lines, default_font_size);
    }
}

fn push_non_whitespace_token(
    token: &str,
    style: &TextStyle,
    max_width: f32,
    current: &mut LineBuilder,
    lines: &mut Vec<WrappedLine>,
    default_font_size: f32,
) {
    let token_width = estimate_text_width(token, style);

    if current.is_empty() {
        if token_width <= max_width {
            current.push_text(token.to_string(), style.clone());
            return;
        }

        for segment in break_text_to_width(token, max_width, style) {
            current.push_text(segment, style.clone());
            flush_line(current, lines, default_font_size, false);
        }
        return;
    }

    if current.width + token_width <= max_width {
        current.push_text(token.to_string(), style.clone());
        return;
    }

    flush_line(current, lines, default_font_size, false);

    if token_width <= max_width {
        current.push_text(token.to_string(), style.clone());
        return;
    }

    for segment in break_text_to_width(token, max_width, style) {
        current.push_text(segment, style.clone());
        flush_line(current, lines, default_font_size, false);
    }
}

fn flush_line(
    current: &mut LineBuilder,
    lines: &mut Vec<WrappedLine>,
    default_font_size: f32,
    force_empty: bool,
) {
    if current.is_empty() {
        if force_empty {
            lines.push(WrappedLine {
                spans: Vec::new(),
                width: 0.0,
                height: (default_font_size * 1.2).max(18.0),
            });
        }
        return;
    }

    let line = std::mem::replace(current, LineBuilder::new()).finish(default_font_size);
    lines.push(line);
}

fn tokenize_text_for_layout(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_is_whitespace: Option<bool> = None;

    for character in text.chars() {
        if is_wide_character(character) {
            if !current.is_empty() {
                tokens.push(current);
                current = String::new();
            }
            tokens.push(character.to_string());
            current_is_whitespace = None;
            continue;
        }

        let is_whitespace = character.is_whitespace();
        match current_is_whitespace {
            Some(flag) if flag == is_whitespace => current.push(character),
            Some(_) => {
                tokens.push(current);
                current = String::from(character);
                current_is_whitespace = Some(is_whitespace);
            }
            None => {
                current.push(character);
                current_is_whitespace = Some(is_whitespace);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn break_text_to_width(text: &str, max_width: f32, style: &TextStyle) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut current_width = 0.0;

    for character in text.chars() {
        let width = estimated_char_width(character, style);
        if !current.is_empty() && current_width + width > max_width {
            segments.push(current);
            current = String::new();
            current_width = 0.0;
        }

        current.push(character);
        current_width += width;
    }

    if !current.is_empty() {
        segments.push(current);
    }

    if segments.is_empty() {
        segments.push(text.to_string());
    }

    segments
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

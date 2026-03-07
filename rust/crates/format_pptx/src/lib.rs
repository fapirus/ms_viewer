use base64::Engine;
use format_shared::{parse_shared_package, resolve_relationship_target};
use viewer_core::archive::OoxmlArchive;
use viewer_core::model::{
    BoxNode, ImageNode, ImageReference, PageRenderModel, ParagraphAlignment, Rect, RenderNode,
    TextNode, TextRange, TextStyle,
};
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub placeholder: Option<SlidePlaceholderKind>,
    pub paragraphs: Vec<SlideTextParagraph>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PptxSlideTree {
    pub presentation_part: String,
    pub presentation_size: Option<PresentationSize>,
    pub slides: Vec<SlideReference>,
    pub slide_masters: Vec<SlideMasterReference>,
}

pub fn parse_pptx(archive: &OoxmlArchive) -> Result<PptxSlideTree, ViewerError> {
    parse_slide_tree(archive)
}

pub fn parse_slide_text_boxes(
    archive: &OoxmlArchive,
    slide_part_name: &str,
) -> Result<Vec<SlideTextBox>, ViewerError> {
    let slide_xml = archive.read_part(slide_part_name)?;
    let slide_text = String::from_utf8(slide_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let slide_root = parse_document(&slide_text)?;

    if slide_root.local_name() != "sld" {
        return Err(ViewerError::InvalidDocument);
    }

    let shape_tree = slide_root
        .child("cSld")
        .and_then(|common_slide| common_slide.child("spTree"))
        .ok_or(ViewerError::InvalidDocument)?;

    let mut text_boxes = Vec::new();
    for child in &shape_tree.children {
        if child.local_name() != "sp" {
            continue;
        }

        let Some(text_box) = parse_text_box_shape(child)? else {
            continue;
        };
        text_boxes.push(text_box);
    }

    Ok(text_boxes)
}

pub fn parse_slide_basic_shapes(
    archive: &OoxmlArchive,
    slide_part_name: &str,
) -> Result<Vec<BasicShape>, ViewerError> {
    let slide_xml = archive.read_part(slide_part_name)?;
    let slide_text = String::from_utf8(slide_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let slide_root = parse_document(&slide_text)?;

    if slide_root.local_name() != "sld" {
        return Err(ViewerError::InvalidDocument);
    }

    let shape_tree = slide_root
        .child("cSld")
        .and_then(|common_slide| common_slide.child("spTree"))
        .ok_or(ViewerError::InvalidDocument)?;

    let mut shapes = Vec::new();
    for child in &shape_tree.children {
        if child.local_name() != "sp" {
            continue;
        }

        let Some(shape) = parse_basic_shape(child)? else {
            continue;
        };
        shapes.push(shape);
    }

    Ok(shapes)
}

pub fn parse_slide_images(
    archive: &OoxmlArchive,
    slide_part_name: &str,
) -> Result<Vec<SlideImage>, ViewerError> {
    let slide_xml = archive.read_part(slide_part_name)?;
    let slide_text = String::from_utf8(slide_xml).map_err(|_| ViewerError::InvalidDocument)?;
    let slide_root = parse_document(&slide_text)?;

    if slide_root.local_name() != "sld" {
        return Err(ViewerError::InvalidDocument);
    }

    let relationships = parse_part_relationships(archive, slide_part_name)?;
    let shape_tree = slide_root
        .child("cSld")
        .and_then(|common_slide| common_slide.child("spTree"))
        .ok_or(ViewerError::InvalidDocument)?;

    let mut images = Vec::new();
    for child in &shape_tree.children {
        if child.local_name() != "pic" {
            continue;
        }

        images.push(parse_slide_image(child, &relationships)?);
    }

    Ok(images)
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

    let mut nodes = Vec::new();

    for shape in parse_slide_basic_shapes(archive, &slide.part_name)? {
        let Some(transform) = shape.transform else {
            continue;
        };

        nodes.push(RenderNode::Box(BoxNode {
            bounds: rect_from_emu_bounds(&transform.bounds),
            fill_color_hex: normalize_render_color(shape.fill.as_ref()),
            stroke_color_hex: normalize_render_stroke_color(shape.stroke.as_ref()),
            stroke_width: shape
                .stroke
                .as_ref()
                .and_then(|stroke| stroke.width_emu)
                .map(emu_to_points)
                .unwrap_or(0.0),
        }));
    }

    for image in parse_slide_images(archive, &slide.part_name)? {
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

    for text_box in parse_slide_text_boxes(archive, &slide.part_name)? {
        nodes.extend(build_text_nodes(&text_box));
    }

    Ok(PageRenderModel {
        page_index: slide_index as u32,
        width: slide_width,
        height: slide_height,
        nodes,
        selection_anchors: Vec::new(),
    })
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

fn parse_text_box_shape(shape: &XmlElement) -> Result<Option<SlideTextBox>, ViewerError> {
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
        .map(parse_shape_transform)
        .transpose()?
        .map(|transform| transform.bounds);
    let insets = text_body
        .child("bodyPr")
        .map(parse_text_insets)
        .transpose()?
        .unwrap_or_default();
    let placeholder = parse_placeholder_kind(non_visual);
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

fn parse_basic_shape(shape: &XmlElement) -> Result<Option<BasicShape>, ViewerError> {
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
        .map(parse_shape_transform)
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

fn parse_slide_image(
    picture: &XmlElement,
    relationships: &[SlideRelationship],
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
        .map(parse_shape_transform)
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

fn parse_shape_transform(transform: &XmlElement) -> Result<ShapeTransform, ViewerError> {
    let offset = transform.child("off").ok_or(ViewerError::InvalidDocument)?;
    let extent = transform.child("ext").ok_or(ViewerError::InvalidDocument)?;

    Ok(ShapeTransform {
        bounds: EmuRectangle {
            x: parse_i64_attribute(offset, "x")?,
            y: parse_i64_attribute(offset, "y")?,
            width: parse_i64_attribute(extent, "cx")?,
            height: parse_i64_attribute(extent, "cy")?,
        },
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

fn parse_placeholder_kind(non_visual: &XmlElement) -> Option<SlidePlaceholderKind> {
    let placeholder = non_visual.child("nvPr")?.child("ph")?;
    let placeholder_type = placeholder.attribute("type").unwrap_or("body");

    Some(match placeholder_type {
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
    })
}

fn parse_shape_fill(shape_properties: &XmlElement) -> Result<Option<ShapeFill>, ViewerError> {
    if shape_properties.child("noFill").is_some() {
        return Ok(Some(ShapeFill::None));
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

fn normalize_render_color(fill: Option<&ShapeFill>) -> Option<String> {
    match fill {
        Some(ShapeFill::Solid(color)) if color.starts_with('#') => Some(color.clone()),
        _ => None,
    }
}

fn normalize_render_stroke_color(stroke: Option<&ShapeStroke>) -> Option<String> {
    let Some(stroke) = stroke else {
        return None;
    };
    if stroke.is_none {
        return None;
    }
    stroke
        .color
        .as_ref()
        .filter(|color| color.starts_with('#'))
        .cloned()
}

fn build_text_nodes(text_box: &SlideTextBox) -> Vec<RenderNode> {
    let Some(bounds) = &text_box.bounds else {
        return Vec::new();
    };

    let left_inset = emu_to_points(text_box.insets.left);
    let top_inset = emu_to_points(text_box.insets.top);
    let right_inset = emu_to_points(text_box.insets.right);
    let bottom_inset = emu_to_points(text_box.insets.bottom);
    let content_width = (emu_to_points(bounds.width) - left_inset - right_inset).max(1.0);
    let base_x = emu_to_points(bounds.x) + left_inset;
    let mut cursor_y = emu_to_points(bounds.y) + top_inset;
    let max_bottom = emu_to_points(bounds.y + bounds.height) - bottom_inset;
    let mut nodes = Vec::new();
    let mut text_offset = 0u32;

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
                let char_count = span.text.chars().count() as u32;
                nodes.push(RenderNode::Text(TextNode {
                    text: span.text,
                    bounds: Rect {
                        x: cursor_x,
                        y: cursor_y,
                        width: span.width.max(1.0),
                        height: line.height,
                    },
                    style: span.style,
                    range: TextRange {
                        start: text_offset,
                        end: text_offset + char_count,
                    },
                }));
                text_offset += char_count;
                cursor_x += span.width;
            }

            cursor_y += line.height;
        }

        if cursor_y + paragraph_spacing > max_bottom {
            break;
        }
        cursor_y += paragraph_spacing;
    }

    nodes
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

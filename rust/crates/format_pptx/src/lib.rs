use format_shared::{parse_shared_package, resolve_relationship_target};
use viewer_core::archive::OoxmlArchive;
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
const THEME_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideTextBox {
    pub shape_id: u32,
    pub name: String,
    pub bounds: Option<EmuRectangle>,
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

        slides.push(SlideReference {
            slide_id,
            relationship_id,
            part_name: relationship.resolved_target.trim_start_matches('/').to_string(),
            layout_part_name: find_single_related_part(
                archive,
                relationship.resolved_target.trim_start_matches('/'),
                SLIDE_LAYOUT_RELATIONSHIP,
            )?,
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

fn find_single_related_part(
    archive: &OoxmlArchive,
    part_name: &str,
    relationship_type: &str,
) -> Result<Option<String>, ViewerError> {
    let relationships = parse_part_relationships(archive, part_name)?;
    Ok(relationships
        .iter()
        .find(|relationship| relationship.relationship_type == relationship_type)
        .map(|relationship| relationship.resolved_target.trim_start_matches('/').to_string()))
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
    let placeholder = parse_placeholder_kind(non_visual);
    let paragraphs = parse_text_paragraphs(text_body)?;

    Ok(Some(SlideTextBox {
        shape_id,
        name,
        bounds,
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

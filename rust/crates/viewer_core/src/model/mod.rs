use serde::{Deserialize, Serialize};

pub type DocumentId = String;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentKind {
    Docx,
    Pptx,
    Xlsx,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenOptions {
    pub password: Option<String>,
    pub prefer_lazy_loading: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            password: None,
            prefer_lazy_loading: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextRun {
    pub text: String,
    pub style: TextStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphMetrics {
    pub alignment: ParagraphAlignment,
    pub line_height: Option<f32>,
    pub spacing_before: f32,
    pub spacing_after: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ParagraphAlignment {
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageReference {
    pub resource_id: String,
    pub description: Option<String>,
    pub content_type: Option<String>,
    pub display_width: Option<f32>,
    pub display_height: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ListKind {
    Bullet,
    Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListMarker {
    pub level: u8,
    pub kind: ListKind,
    pub num_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TableCellMerge {
    Restart,
    Continue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TableCell {
    pub blocks: Vec<Block>,
    pub column_span: u16,
    pub row_merge: Option<TableCellMerge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TableAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TableAnchor {
    Margin,
    Page,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TableHorizontalPosition {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TableVerticalPosition {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FloatingTablePosition {
    pub horz_anchor: TableAnchor,
    pub vert_anchor: TableAnchor,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub x_position: Option<TableHorizontalPosition>,
    pub y_position: Option<TableVerticalPosition>,
    pub left_from_text: f32,
    pub right_from_text: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TableLayout {
    pub preferred_width: Option<f32>,
    pub alignment: TableAlignment,
    pub floating: Option<FloatingTablePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Block {
    Paragraph {
        runs: Vec<TextRun>,
        list: Option<ListMarker>,
        metrics: ParagraphMetrics,
    },
    Table {
        rows: Vec<TableRow>,
        column_widths: Vec<f32>,
        layout: TableLayout,
    },
    Image {
        image: ImageReference,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextNode {
    pub text: String,
    pub bounds: Rect,
    pub style: TextStyle,
    pub range: TextRange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageNode {
    pub resource_id: String,
    pub description: Option<String>,
    pub content_type: Option<String>,
    pub data_base64: Option<String>,
    pub bounds: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BoxNode {
    pub bounds: Rect,
    pub fill_color_hex: Option<String>,
    pub stroke_color_hex: Option<String>,
    pub stroke_width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RenderNode {
    Text(TextNode),
    Image(ImageNode),
    Box(BoxNode),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SelectionAnchor {
    pub node_index: u32,
    pub char_index: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PageRenderModel {
    pub page_index: u32,
    pub width: f32,
    pub height: f32,
    pub nodes: Vec<RenderNode>,
    pub selection_anchors: Vec<SelectionAnchor>,
}

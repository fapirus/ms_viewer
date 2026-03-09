use std::path::Path;

use base64::Engine;
use format_docx::{build_selection_page_models, parse_docx, search_document};
use format_pptx::{build_slide_render_model, parse_pptx, search_slides};
use format_xlsx::{
    build_sheet_render_model, build_visible_window_render_model, parse_cell_style_subset,
    parse_shared_strings, parse_xlsx, search_workbook, XlsxVisibleWindow,
};
use serde::{Deserialize, Serialize};
use viewer_core::archive::OoxmlArchive;
use viewer_core::crypto::{
    detect_package_kind_from_bytes, detect_package_kind_from_path, PackageKind,
};
use viewer_core::model::{DocumentKind, OpenOptions, PageRenderModel};
use viewer_core::search::SearchMatch;
pub use viewer_core::wire::{ErrorResponse, ViewerErrorCode};
use viewer_core::xml::parse_document;
use viewer_core::ViewerError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum DocumentSource {
    Path(String),
    BytesBase64(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenDocumentRequest {
    pub source: DocumentSource,
    pub options: OpenOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentCapabilities {
    pub search: bool,
    pub text_selection: bool,
    pub password_protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenDocumentSuccess {
    pub document_id: String,
    pub kind: DocumentKind,
    pub title: String,
    pub page_count: u32,
    pub capabilities: DocumentCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetPageRenderModelRequest {
    pub source: DocumentSource,
    pub document_id: String,
    pub page_index: u32,
    #[serde(default)]
    pub sheet_window: Option<SheetWindow>,
    pub options: OpenOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SheetWindow {
    pub start_row: u32,
    pub end_row: u32,
    pub start_column: u32,
    pub end_column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchDocumentRequest {
    pub source: DocumentSource,
    pub document_id: String,
    pub query: String,
    pub options: OpenOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetSelectionPageRequest {
    pub source: DocumentSource,
    pub document_id: String,
    pub page_index: u32,
    pub options: OpenOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum OpenDocumentResponse {
    Success(OpenDocumentSuccess),
    Error(ErrorResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GetPageRenderModelResponse {
    Success(PageRenderModel),
    Error(ErrorResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SearchDocumentResponse {
    Success(Vec<SearchMatch>),
    Error(ErrorResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GetSelectionPageResponse {
    Success(PageRenderModel),
    Error(ErrorResponse),
}

pub fn open_document(request: OpenDocumentRequest) -> OpenDocumentResponse {
    match open_document_impl(request) {
        Ok(success) => OpenDocumentResponse::Success(success),
        Err(error) => OpenDocumentResponse::Error(error.to_error_response()),
    }
}

pub fn open_document_json(request_json: &str) -> OpenDocumentResponse {
    let request: OpenDocumentRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(_) => {
            return OpenDocumentResponse::Error(ViewerError::InvalidDocument.to_error_response())
        }
    };

    open_document(request)
}

pub fn get_page_render_model(request: GetPageRenderModelRequest) -> GetPageRenderModelResponse {
    match get_page_render_model_impl(request) {
        Ok(model) => GetPageRenderModelResponse::Success(model),
        Err(error) => GetPageRenderModelResponse::Error(error.to_error_response()),
    }
}

pub fn get_page_render_model_json(request_json: &str) -> GetPageRenderModelResponse {
    let request: GetPageRenderModelRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(_) => {
            return GetPageRenderModelResponse::Error(
                ViewerError::InvalidDocument.to_error_response(),
            )
        }
    };

    get_page_render_model(request)
}

pub fn search_document_pages(request: SearchDocumentRequest) -> SearchDocumentResponse {
    match search_document_pages_impl(request) {
        Ok(matches) => SearchDocumentResponse::Success(matches),
        Err(error) => SearchDocumentResponse::Error(error.to_error_response()),
    }
}

pub fn search_document_pages_json(request_json: &str) -> SearchDocumentResponse {
    let request: SearchDocumentRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(_) => {
            return SearchDocumentResponse::Error(ViewerError::InvalidDocument.to_error_response())
        }
    };

    search_document_pages(request)
}

pub fn get_selection_page(request: GetSelectionPageRequest) -> GetSelectionPageResponse {
    match get_selection_page_impl(request) {
        Ok(page) => GetSelectionPageResponse::Success(page),
        Err(error) => GetSelectionPageResponse::Error(error.to_error_response()),
    }
}

pub fn get_selection_page_json(request_json: &str) -> GetSelectionPageResponse {
    let request: GetSelectionPageRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(_) => {
            return GetSelectionPageResponse::Error(
                ViewerError::InvalidDocument.to_error_response(),
            )
        }
    };

    get_selection_page(request)
}

fn open_document_impl(request: OpenDocumentRequest) -> Result<OpenDocumentSuccess, ViewerError> {
    let source = request.source.clone();
    let title = source_title(&source);

    match source {
        DocumentSource::Path(path) => open_document_from_path(
            &path,
            &DocumentSource::Path(path.clone()),
            title,
            &request.options,
        ),
        DocumentSource::BytesBase64(value) => {
            open_document_from_bytes(value, &request.source, title, &request.options)
        }
    }
}

fn open_document_from_path(
    path: &str,
    source: &DocumentSource,
    title: String,
    options: &OpenOptions,
) -> Result<OpenDocumentSuccess, ViewerError> {
    match detect_package_kind_from_path(path)? {
        PackageKind::Plain => {
            let archive = OoxmlArchive::open_path(path)?;
            build_open_success(&archive, source, title)
        }
        PackageKind::Encrypted => encrypted_error(options),
        PackageKind::UnsupportedEncryption => Err(ViewerError::UnsupportedEncryption),
    }
}

fn open_document_from_bytes(
    value: String,
    source: &DocumentSource,
    title: String,
    options: &OpenOptions,
) -> Result<OpenDocumentSuccess, ViewerError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|_| ViewerError::InvalidDocument)?;
    match detect_package_kind_from_bytes(&bytes)? {
        PackageKind::Plain => {
            let archive = OoxmlArchive::open_bytes(bytes)?;
            build_open_success(&archive, source, title)
        }
        PackageKind::Encrypted => encrypted_error(options),
        PackageKind::UnsupportedEncryption => Err(ViewerError::UnsupportedEncryption),
    }
}

fn encrypted_error(options: &OpenOptions) -> Result<OpenDocumentSuccess, ViewerError> {
    if options.password.is_some() {
        Err(ViewerError::UnsupportedEncryption)
    } else {
        Err(ViewerError::PasswordRequired)
    }
}

fn build_open_success(
    archive: &OoxmlArchive,
    source: &DocumentSource,
    title: String,
) -> Result<OpenDocumentSuccess, ViewerError> {
    let kind = detect_document_kind(archive)?;
    let page_count = detect_page_count(archive, kind)?;
    let document_id = build_document_id(source, &title);

    Ok(OpenDocumentSuccess {
        document_id,
        kind,
        title,
        page_count,
        capabilities: DocumentCapabilities {
            search: supports_text_search(kind),
            text_selection: supports_text_selection(kind),
            password_protected: false,
        },
    })
}

fn detect_document_kind(archive: &OoxmlArchive) -> Result<DocumentKind, ViewerError> {
    if archive.contains_part("word/document.xml") {
        return Ok(DocumentKind::Docx);
    }
    if archive.contains_part("ppt/presentation.xml") {
        return Ok(DocumentKind::Pptx);
    }
    if archive.contains_part("xl/workbook.xml") {
        return Ok(DocumentKind::Xlsx);
    }
    Err(ViewerError::UnsupportedFormat)
}

fn detect_page_count(archive: &OoxmlArchive, kind: DocumentKind) -> Result<u32, ViewerError> {
    match kind {
        DocumentKind::Docx => {
            let package = parse_docx(archive)?;
            let pages = build_selection_page_models(archive, &package)?;
            Ok(pages.len().max(1) as u32)
        }
        DocumentKind::Pptx => count_xml_elements(archive, "ppt/presentation.xml", "sldId"),
        DocumentKind::Xlsx => count_xml_elements(archive, "xl/workbook.xml", "sheet"),
    }
}

fn count_xml_elements(
    archive: &OoxmlArchive,
    part_name: &str,
    element_name: &str,
) -> Result<u32, ViewerError> {
    let bytes = archive.read_part(part_name)?;
    let text = String::from_utf8(bytes).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;

    Ok(root
        .children
        .iter()
        .flat_map(|child| child.children.iter().chain(std::iter::once(child)))
        .filter(|child| child.local_name() == element_name)
        .count() as u32)
}

fn source_title(source: &DocumentSource) -> String {
    match source {
        DocumentSource::Path(path) => Path::new(path)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("document")
            .to_string(),
        DocumentSource::BytesBase64(_) => "memory-document".to_string(),
    }
}

fn build_document_id(source: &DocumentSource, title: &str) -> String {
    match source {
        DocumentSource::Path(path) => format!("path:{path}"),
        DocumentSource::BytesBase64(_) => format!("memory:{title}"),
    }
}

fn get_page_render_model_impl(
    request: GetPageRenderModelRequest,
) -> Result<PageRenderModel, ViewerError> {
    let _ = &request.document_id;

    let archive = open_archive_from_source(request.source)?;
    match detect_document_kind(&archive)? {
        DocumentKind::Docx => {
            let package = parse_docx(&archive)?;
            let pages = build_selection_page_models(&archive, &package)?;
            pages
                .get(request.page_index as usize)
                .cloned()
                .ok_or(ViewerError::InvalidDocument)
        }
        DocumentKind::Pptx => {
            let slide_tree = parse_pptx(&archive)?;
            build_slide_render_model(&archive, &slide_tree, request.page_index as usize)
        }
        DocumentKind::Xlsx => {
            let workbook = parse_xlsx(&archive)?;
            let shared_strings = parse_shared_strings(&archive, &workbook)?;
            let styles = parse_cell_style_subset(&archive, &workbook)?;
            if let Some(window) = request.sheet_window.as_ref() {
                if window.end_row < window.start_row || window.end_column < window.start_column {
                    return Err(ViewerError::InvalidDocument);
                }
                build_visible_window_render_model(
                    &archive,
                    &workbook,
                    &shared_strings,
                    &styles,
                    request.page_index as usize,
                    &XlsxVisibleWindow {
                        start_row: window.start_row,
                        start_column: window.start_column,
                        row_count: window.end_row - window.start_row + 1,
                        column_count: window.end_column - window.start_column + 1,
                    },
                )
            } else {
                build_sheet_render_model(
                    &archive,
                    &workbook,
                    &shared_strings,
                    &styles,
                    request.page_index as usize,
                )
            }
        }
    }
}

fn search_document_pages_impl(
    request: SearchDocumentRequest,
) -> Result<Vec<SearchMatch>, ViewerError> {
    let _ = &request.document_id;

    let archive = open_archive_from_source(request.source)?;
    match detect_document_kind(&archive)? {
        DocumentKind::Docx => {
            let package = parse_docx(&archive)?;
            search_document(&archive, &package, &request.query)
        }
        DocumentKind::Pptx => {
            let slide_tree = parse_pptx(&archive)?;
            search_slides(&archive, &slide_tree, &request.query)
        }
        DocumentKind::Xlsx => {
            let workbook = parse_xlsx(&archive)?;
            let shared_strings = parse_shared_strings(&archive, &workbook)?;
            let styles = parse_cell_style_subset(&archive, &workbook)?;
            search_workbook(
                &archive,
                &workbook,
                &shared_strings,
                &styles,
                &request.query,
            )
        }
    }
}

fn get_selection_page_impl(
    request: GetSelectionPageRequest,
) -> Result<PageRenderModel, ViewerError> {
    get_page_render_model_impl(GetPageRenderModelRequest {
        source: request.source,
        document_id: request.document_id,
        page_index: request.page_index,
        sheet_window: None,
        options: request.options,
    })
}

fn open_archive_from_source(source: DocumentSource) -> Result<OoxmlArchive, ViewerError> {
    match source {
        DocumentSource::Path(path) => OoxmlArchive::open_path(&path),
        DocumentSource::BytesBase64(value) => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(value)
                .map_err(|_| ViewerError::InvalidDocument)?;
            OoxmlArchive::open_bytes(bytes)
        }
    }
}

fn supports_text_search(kind: DocumentKind) -> bool {
    matches!(
        kind,
        DocumentKind::Docx | DocumentKind::Pptx | DocumentKind::Xlsx
    )
}

fn supports_text_selection(kind: DocumentKind) -> bool {
    matches!(
        kind,
        DocumentKind::Docx | DocumentKind::Pptx | DocumentKind::Xlsx
    )
}

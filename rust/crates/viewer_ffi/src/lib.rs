use std::path::Path;

use base64::Engine;
use format_docx::{build_selection_page_models, parse_docx};
use serde::{Deserialize, Serialize};
use viewer_core::archive::OoxmlArchive;
use viewer_core::crypto::{
    detect_package_kind_from_bytes, detect_package_kind_from_path, PackageKind,
};
use viewer_core::model::{DocumentKind, OpenOptions, PageRenderModel};
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

fn open_document_impl(request: OpenDocumentRequest) -> Result<OpenDocumentSuccess, ViewerError> {
    let source = request.source.clone();
    let title = source_title(&source);

    match source {
        DocumentSource::Path(path) => {
            open_document_from_path(&path, &DocumentSource::Path(path.clone()), title, &request.options)
        }
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
            search: kind == DocumentKind::Docx,
            text_selection: kind == DocumentKind::Docx,
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

    match request.source {
        DocumentSource::Path(path) => {
            let archive = OoxmlArchive::open_path(&path)?;
            let package = parse_docx(&archive)?;
            let pages = build_selection_page_models(&archive, &package)?;
            pages
                .get(request.page_index as usize)
                .cloned()
                .ok_or(ViewerError::InvalidDocument)
        }
        DocumentSource::BytesBase64(value) => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(value)
                .map_err(|_| ViewerError::InvalidDocument)?;
            let archive = OoxmlArchive::open_bytes(bytes)?;
            let package = parse_docx(&archive)?;
            let pages = build_selection_page_models(&archive, &package)?;
            pages
                .get(request.page_index as usize)
                .cloned()
                .ok_or(ViewerError::InvalidDocument)
        }
    }
}

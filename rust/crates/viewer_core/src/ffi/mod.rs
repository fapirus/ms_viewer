use std::path::Path;

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::archive::OoxmlArchive;
use crate::crypto::{
    detect_package_kind_from_bytes, detect_package_kind_from_path, PackageKind,
};
use crate::error::ViewerError;
use crate::model::{DocumentKind, OpenOptions};
use crate::xml::parse_document;

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
#[serde(rename_all = "snake_case")]
pub enum ViewerErrorCode {
    UnsupportedFormat,
    PasswordRequired,
    InvalidPassword,
    UnsupportedEncryption,
    IoError,
    InvalidDocument,
    NotImplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub code: ViewerErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum OpenDocumentResponse {
    Success(OpenDocumentSuccess),
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

fn open_document_impl(request: OpenDocumentRequest) -> Result<OpenDocumentSuccess, ViewerError> {
    let title = source_title(&request.source);

    match request.source {
        DocumentSource::Path(path) => open_document_from_path(&path, title, &request.options),
        DocumentSource::BytesBase64(value) => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(value)
                .map_err(|_| ViewerError::InvalidDocument)?;
            open_document_from_bytes(bytes, title, &request.options)
        }
    }
}

fn open_document_from_path(
    path: &str,
    title: String,
    options: &OpenOptions,
) -> Result<OpenDocumentSuccess, ViewerError> {
    match detect_package_kind_from_path(path)? {
        PackageKind::Plain => {
            let archive = OoxmlArchive::open_path(path)?;
            build_open_success(&archive, title)
        }
        PackageKind::Encrypted => encrypted_error(options),
        PackageKind::UnsupportedEncryption => Err(ViewerError::UnsupportedEncryption),
    }
}

fn open_document_from_bytes(
    bytes: Vec<u8>,
    title: String,
    options: &OpenOptions,
) -> Result<OpenDocumentSuccess, ViewerError> {
    match detect_package_kind_from_bytes(&bytes)? {
        PackageKind::Plain => {
            let archive = OoxmlArchive::open_bytes(bytes)?;
            build_open_success(&archive, title)
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
    title: String,
) -> Result<OpenDocumentSuccess, ViewerError> {
    let kind = detect_document_kind(archive)?;
    let page_count = detect_page_count(archive, kind)?;

    Ok(OpenDocumentSuccess {
        document_id: format!("session:{}", title),
        kind,
        title,
        page_count,
        capabilities: DocumentCapabilities {
            search: false,
            text_selection: false,
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
        DocumentKind::Docx => Ok(0),
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

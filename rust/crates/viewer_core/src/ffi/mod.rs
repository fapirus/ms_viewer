use serde::{Deserialize, Serialize};

use crate::model::{DocumentKind, OpenOptions};

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

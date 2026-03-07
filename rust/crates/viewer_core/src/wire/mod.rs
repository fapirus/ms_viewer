use serde::{Deserialize, Serialize};

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

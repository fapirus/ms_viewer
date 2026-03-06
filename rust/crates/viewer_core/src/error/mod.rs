use crate::ffi::{ErrorResponse, ViewerErrorCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ViewerError {
    #[error("unsupported format")]
    UnsupportedFormat,
    #[error("password required")]
    PasswordRequired,
    #[error("invalid password")]
    InvalidPassword,
    #[error("unsupported encryption")]
    UnsupportedEncryption,
    #[error("invalid document")]
    InvalidDocument,
    #[error("io error: {0}")]
    Io(String),
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}

impl ViewerError {
    pub fn code(&self) -> ViewerErrorCode {
        match self {
            Self::UnsupportedFormat => ViewerErrorCode::UnsupportedFormat,
            Self::PasswordRequired => ViewerErrorCode::PasswordRequired,
            Self::InvalidPassword => ViewerErrorCode::InvalidPassword,
            Self::UnsupportedEncryption => ViewerErrorCode::UnsupportedEncryption,
            Self::InvalidDocument => ViewerErrorCode::InvalidDocument,
            Self::Io(_) => ViewerErrorCode::IoError,
            Self::NotImplemented(_) => ViewerErrorCode::NotImplemented,
        }
    }

    pub fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse {
            code: self.code(),
            message: self.to_string(),
        }
    }
}

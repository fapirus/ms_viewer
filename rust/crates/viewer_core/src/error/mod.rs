use thiserror::Error;

#[derive(Debug, Error)]
pub enum ViewerError {
    #[error("unsupported format")]
    UnsupportedFormat,
    #[error("password required")]
    PasswordRequired,
    #[error("invalid password")]
    InvalidPassword,
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}

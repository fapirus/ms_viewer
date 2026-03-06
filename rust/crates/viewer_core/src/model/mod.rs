use serde::{Deserialize, Serialize};

pub type DocumentId = String;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentKind {
    Docx,
    Pptx,
    Xlsx,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

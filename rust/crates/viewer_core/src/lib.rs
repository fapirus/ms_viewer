pub mod archive;
pub mod cache;
pub mod crypto;
pub mod error;
pub mod layout;
pub mod model;
pub mod search;
pub mod text;
pub mod wire;
pub mod xml;

pub use error::ViewerError;
pub use model::{DocumentId, DocumentKind, OpenOptions};

pub use crypto::{detect_package_kind_from_bytes, detect_package_kind_from_path, PackageKind};

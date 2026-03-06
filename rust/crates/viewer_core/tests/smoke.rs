use viewer_core::{DocumentKind, OpenOptions};

#[test]
fn default_open_options_enable_lazy_loading() {
    let options = OpenOptions::default();
    assert!(options.prefer_lazy_loading);
    assert_eq!(DocumentKind::Docx as u8, 0);
}

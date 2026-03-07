use format_docx::{build_selection_page_models, layout_document, parse_docx, search_document};
use std::path::PathBuf;
use viewer_core::archive::OoxmlArchive;
use viewer_core::OpenOptions;
use viewer_ffi::{open_document, DocumentSource, OpenDocumentRequest, OpenDocumentResponse};

fn fixture_path(relative: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(relative)
        .to_string_lossy()
        .to_string()
}

#[test]
fn acceptance_document_opens_from_review_fixture() {
    let archive = OoxmlArchive::open_path(fixture_path("fixtures/docx/docx_plain_text.docx"))
        .expect("fixture archive");
    let package = parse_docx(&archive).expect("docx package");

    assert_eq!(package.main_document, "word/document.xml");
}

#[test]
fn acceptance_page_rendering_works_for_review_fixture() {
    let archive = OoxmlArchive::open_path(fixture_path("fixtures/docx/docx_tables_images.docx"))
        .expect("fixture archive");
    let package = parse_docx(&archive).expect("docx package");
    let pages = layout_document(&archive, &package).expect("layout");

    assert!(!pages.is_empty());
    assert!(!pages[0].blocks.is_empty());
}

#[test]
fn acceptance_search_works_for_review_fixture() {
    let archive = OoxmlArchive::open_path(fixture_path("fixtures/docx/docx_styles_lists.docx"))
        .expect("fixture archive");
    let package = parse_docx(&archive).expect("docx package");
    let matches = search_document(&archive, &package, "bullet").expect("search");

    assert!(!matches.is_empty());
    assert_eq!(matches[0].page_index, 0);
}

#[test]
fn acceptance_text_selection_works_for_review_fixture() {
    let archive = OoxmlArchive::open_path(fixture_path("fixtures/docx/docx_plain_text.docx"))
        .expect("fixture archive");
    let package = parse_docx(&archive).expect("docx package");
    let pages = build_selection_page_models(&archive, &package).expect("selection models");

    assert!(!pages.is_empty());
    assert!(!pages[0].selection_anchors.is_empty());
    assert!(!pages[0].nodes.is_empty());
}

#[test]
fn acceptance_encrypted_document_asks_for_password() {
    let response = open_document(OpenDocumentRequest {
        source: DocumentSource::Path(fixture_path("fixtures/encrypted/docx_password_stub.docx")),
        options: OpenOptions::default(),
    });

    match response {
        OpenDocumentResponse::Error(error) => {
            assert_eq!(
                error.code,
                viewer_core::wire::ViewerErrorCode::PasswordRequired
            );
        }
        other => panic!("expected password required error, got {other:?}"),
    }
}

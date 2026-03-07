use std::fs;
use std::io::Write;
use std::path::PathBuf;

use base64::Engine;
use tempfile::NamedTempFile;
use viewer_core::OpenOptions;
use viewer_ffi::{
    get_page_render_model, get_selection_page, open_document, open_document_json,
    search_document_pages, DocumentSource, GetPageRenderModelRequest, GetPageRenderModelResponse,
    GetSelectionPageRequest, GetSelectionPageResponse, OpenDocumentRequest, OpenDocumentResponse,
    SearchDocumentRequest, SearchDocumentResponse,
};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[test]
fn opens_docx_path_request_and_returns_basic_metadata() {
    let file = create_package(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello DOCX render</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>"#,
        ),
    ]);
    let path = file.path().to_string_lossy().to_string();

    let response = open_document(OpenDocumentRequest {
        source: DocumentSource::Path(path.clone()),
        options: OpenOptions::default(),
    });

    match response {
        OpenDocumentResponse::Success(success) => {
            assert_eq!(success.kind, viewer_core::DocumentKind::Docx);
            assert_eq!(
                success.title,
                file.path().file_name().unwrap().to_string_lossy()
            );
            assert_eq!(success.page_count, 1);
            assert!(success.capabilities.search);
            assert!(success.capabilities.text_selection);
        }
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn opens_rendered_page_break_regression_fixture_with_two_pages() {
    let path = regression_fixture_path("docx_rendered_page_break.docx");
    let document_id = format!("path:{}", path.display());

    let open_response = open_document(OpenDocumentRequest {
        source: DocumentSource::Path(path.display().to_string()),
        options: OpenOptions::default(),
    });

    match open_response {
        OpenDocumentResponse::Success(success) => {
            assert_eq!(success.kind, viewer_core::DocumentKind::Docx);
            assert_eq!(success.page_count, 2);
        }
        other => panic!("expected success, got {other:?}"),
    }

    let first_page = get_page_render_model(GetPageRenderModelRequest {
        source: DocumentSource::Path(path.display().to_string()),
        document_id: document_id.clone(),
        page_index: 0,
        options: OpenOptions::default(),
    });
    let second_page = get_page_render_model(GetPageRenderModelRequest {
        source: DocumentSource::Path(path.display().to_string()),
        document_id,
        page_index: 1,
        options: OpenOptions::default(),
    });

    match first_page {
        GetPageRenderModelResponse::Success(page) => {
            let texts = page
                .nodes
                .iter()
                .filter_map(|node| match node {
                    viewer_core::model::RenderNode::Text(text) => Some(text.text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(texts, vec!["alpha"]);
        }
        other => panic!("expected first page, got {other:?}"),
    }

    match second_page {
        GetPageRenderModelResponse::Success(page) => {
            let texts = page
                .nodes
                .iter()
                .filter_map(|node| match node {
                    viewer_core::model::RenderNode::Text(text) => Some(text.text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(texts, vec!["beta"]);
        }
        other => panic!("expected second page, got {other:?}"),
    }
}

#[test]
fn table_cell_layout_regression_fixture_preserves_cell_text_style() {
    let path = regression_fixture_path("docx_table_cell_layout.docx");
    let response = get_page_render_model(GetPageRenderModelRequest {
        source: DocumentSource::Path(path.display().to_string()),
        document_id: format!("path:{}", path.display()),
        page_index: 0,
        options: OpenOptions::default(),
    });

    match response {
        GetPageRenderModelResponse::Success(page) => {
            let styled = page
                .nodes
                .iter()
                .filter_map(|node| match node {
                    viewer_core::model::RenderNode::Text(text) => Some(text),
                    _ => None,
                })
                .find(|text| text.text.contains("Styled"))
                .expect("styled table text should exist");
            let second = page
                .nodes
                .iter()
                .filter_map(|node| match node {
                    viewer_core::model::RenderNode::Text(text) => Some(text),
                    _ => None,
                })
                .find(|text| text.text.contains("Second"))
                .expect("second paragraph text should exist");

            assert!(styled.style.bold);
            assert_eq!(styled.style.font_size, 16.0);
            assert!(second.bounds.y > styled.bounds.y);
        }
        other => panic!("expected page render model, got {other:?}"),
    }
}

#[test]
fn opens_xlsx_bytes_request_from_json() {
    let file = create_package(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="officeDocument" Target="xl/workbook.xml"/>
            </Relationships>"#,
        ),
        (
            "xl/workbook.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
              <sheets>
                <sheet name="Sheet1" sheetId="1" r:id="rId1" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/>
                <sheet name="Sheet2" sheetId="2" r:id="rId2" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/>
              </sheets>
            </workbook>"#,
        ),
    ]);
    let bytes = fs::read(file.path()).expect("fixture bytes");
    let json = serde_json::json!({
        "source": {
            "kind": "bytesBase64",
            "value": base64::engine::general_purpose::STANDARD.encode(bytes),
        },
        "options": {
            "password": null,
            "preferLazyLoading": true,
        }
    })
    .to_string();

    let response = open_document_json(&json);

    match response {
        OpenDocumentResponse::Success(success) => {
            assert_eq!(success.kind, viewer_core::DocumentKind::Xlsx);
            assert_eq!(success.title, "memory-document");
            assert_eq!(success.page_count, 2);
        }
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn encrypted_document_requires_password_first() {
    let file = NamedTempFile::new().expect("temp file");
    fs::write(file.path(), fake_encrypted_ole_bytes()).expect("write encrypted stub");

    let response = open_document(OpenDocumentRequest {
        source: DocumentSource::Path(file.path().to_string_lossy().to_string()),
        options: OpenOptions::default(),
    });

    match response {
        OpenDocumentResponse::Error(error) => {
            assert_eq!(
                error.code,
                viewer_core::wire::ViewerErrorCode::PasswordRequired
            );
        }
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn encrypted_document_with_password_is_reported_as_unsupported_for_now() {
    let file = NamedTempFile::new().expect("temp file");
    fs::write(file.path(), fake_encrypted_ole_bytes()).expect("write encrypted stub");

    let response = open_document(OpenDocumentRequest {
        source: DocumentSource::Path(file.path().to_string_lossy().to_string()),
        options: OpenOptions {
            password: Some("secret".to_string()),
            prefer_lazy_loading: true,
        },
    });

    match response {
        OpenDocumentResponse::Error(error) => {
            assert_eq!(
                error.code,
                viewer_core::wire::ViewerErrorCode::UnsupportedEncryption
            );
        }
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn fetches_first_docx_page_render_model() {
    let file = create_package(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p><w:r><w:t>Hello DOCX render</w:t></w:r></w:p>
                <w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>
              </w:body>
            </w:document>"#,
        ),
    ]);
    let path = file.path().to_string_lossy().to_string();
    let document_id = format!("path:{path}");

    let response = get_page_render_model(GetPageRenderModelRequest {
        source: DocumentSource::Path(path),
        document_id,
        page_index: 0,
        options: OpenOptions::default(),
    });

    match response {
        GetPageRenderModelResponse::Success(page) => {
            assert_eq!(page.page_index, 0);
            assert!(!page.nodes.is_empty());
            assert!(!page.selection_anchors.is_empty());
        }
        other => panic!("expected page render model, got {other:?}"),
    }
}

#[test]
fn invalid_page_index_maps_to_invalid_document_error() {
    let file = create_package(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p><w:r><w:t>Hello DOCX render</w:t></w:r></w:p>
                <w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>
              </w:body>
            </w:document>"#,
        ),
    ]);
    let path = file.path().to_string_lossy().to_string();

    let response = get_page_render_model(GetPageRenderModelRequest {
        source: DocumentSource::Path(path),
        document_id: "unused".to_string(),
        page_index: 9,
        options: OpenOptions::default(),
    });

    match response {
        GetPageRenderModelResponse::Error(error) => {
            assert_eq!(
                error.code,
                viewer_core::wire::ViewerErrorCode::InvalidDocument
            );
        }
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn search_query_returns_docx_matches() {
    let file = create_package(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p><w:r><w:t>Hello searchable DOCX render</w:t></w:r></w:p>
                <w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>
              </w:body>
            </w:document>"#,
        ),
    ]);
    let path = file.path().to_string_lossy().to_string();

    let response = search_document_pages(SearchDocumentRequest {
        source: DocumentSource::Path(path),
        document_id: "unused".to_string(),
        query: "searchable".to_string(),
        options: OpenOptions::default(),
    });

    match response {
        SearchDocumentResponse::Success(matches) => {
            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0].page_index, 0);
            assert_eq!(matches[0].query, "searchable");
            assert!(matches[0].preview.contains("searchable"));
        }
        other => panic!("expected search matches, got {other:?}"),
    }
}

#[test]
fn selection_metadata_fetch_returns_page_with_anchors() {
    let file = create_package(&[
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
              <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
            </Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
              <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
            </Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p><w:r><w:t>Anchor smoke test</w:t></w:r></w:p>
                <w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>
              </w:body>
            </w:document>"#,
        ),
    ]);
    let path = file.path().to_string_lossy().to_string();

    let response = get_selection_page(GetSelectionPageRequest {
        source: DocumentSource::Path(path),
        document_id: "unused".to_string(),
        page_index: 0,
        options: OpenOptions::default(),
    });

    match response {
        GetSelectionPageResponse::Success(page) => {
            assert_eq!(page.page_index, 0);
            assert!(!page.selection_anchors.is_empty());
        }
        other => panic!("expected selection page, got {other:?}"),
    }
}

fn create_package(entries: &[(&str, &str)]) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("temp zip");
    {
        let writer = file.as_file_mut();
        let mut zip = ZipWriter::new(writer);

        for (name, contents) in entries {
            zip.start_file(*name, SimpleFileOptions::default())
                .expect("zip entry should start");
            zip.write_all(contents.as_bytes())
                .expect("zip entry should write");
        }

        zip.finish().expect("zip should finish");
    }

    file
}

fn regression_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fixtures/regression")
        .join(name)
}

fn fake_encrypted_ole_bytes() -> Vec<u8> {
    let mut bytes = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    bytes.extend_from_slice(b"EncryptionInfo");
    bytes.extend_from_slice(b"EncryptedPackage");
    bytes
}

use viewer_core::ffi::{
    DocumentCapabilities, DocumentSource, OpenDocumentRequest, OpenDocumentSuccess,
    ViewerErrorCode,
};
use viewer_core::{DocumentKind, OpenOptions};

#[test]
fn open_document_request_serializes_to_expected_shape() {
    let request = OpenDocumentRequest {
        source: DocumentSource::Path("/tmp/sample.docx".to_string()),
        options: OpenOptions::default(),
    };

    let value = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(value["source"]["kind"], "path");
    assert_eq!(value["source"]["value"], "/tmp/sample.docx");
    assert_eq!(value["options"]["preferLazyLoading"], true);
}

#[test]
fn open_document_success_round_trips() {
    let response = OpenDocumentSuccess {
        document_id: "doc_001".to_string(),
        kind: DocumentKind::Docx,
        title: "sample.docx".to_string(),
        page_count: 12,
        capabilities: DocumentCapabilities {
            search: true,
            text_selection: true,
            password_protected: false,
        },
    };

    let json = serde_json::to_string(&response).expect("response should serialize");
    let decoded: OpenDocumentSuccess =
        serde_json::from_str(&json).expect("response should deserialize");

    assert_eq!(decoded, response);
}

#[test]
fn error_code_serializes_in_snake_case() {
    let json = serde_json::to_string(&ViewerErrorCode::PasswordRequired)
        .expect("error code should serialize");

    assert_eq!(json, "\"password_required\"");
}

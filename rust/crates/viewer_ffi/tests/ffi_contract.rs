use viewer_ffi::{
    DocumentCapabilities, DocumentSource, ErrorResponse, GetPageRenderModelRequest,
    GetSelectionPageRequest, OpenDocumentRequest, OpenDocumentSuccess, SearchDocumentRequest,
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

#[test]
fn error_response_serializes_with_wire_error_code() {
    let response = ErrorResponse {
        code: ViewerErrorCode::InvalidDocument,
        message: "Broken OOXML package.".to_string(),
    };

    let value = serde_json::to_value(&response).expect("error response should serialize");

    assert_eq!(value["code"], "invalid_document");
    assert_eq!(value["message"], "Broken OOXML package.");
}

#[test]
fn get_page_render_model_request_serializes_to_expected_shape() {
    let request = GetPageRenderModelRequest {
        source: DocumentSource::Path("/tmp/sample.docx".to_string()),
        document_id: "path:/tmp/sample.docx".to_string(),
        page_index: 2,
        options: OpenOptions::default(),
    };

    let value = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(value["source"]["kind"], "path");
    assert_eq!(value["documentId"], "path:/tmp/sample.docx");
    assert_eq!(value["pageIndex"], 2);
}

#[test]
fn search_document_request_serializes_to_expected_shape() {
    let request = SearchDocumentRequest {
        source: DocumentSource::Path("/tmp/sample.docx".to_string()),
        document_id: "path:/tmp/sample.docx".to_string(),
        query: "hello".to_string(),
        options: OpenOptions::default(),
    };

    let value = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(value["source"]["kind"], "path");
    assert_eq!(value["documentId"], "path:/tmp/sample.docx");
    assert_eq!(value["query"], "hello");
}

#[test]
fn get_selection_page_request_serializes_to_expected_shape() {
    let request = GetSelectionPageRequest {
        source: DocumentSource::Path("/tmp/sample.docx".to_string()),
        document_id: "path:/tmp/sample.docx".to_string(),
        page_index: 0,
        options: OpenOptions::default(),
    };

    let value = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(value["source"]["kind"], "path");
    assert_eq!(value["documentId"], "path:/tmp/sample.docx");
    assert_eq!(value["pageIndex"], 0);
}

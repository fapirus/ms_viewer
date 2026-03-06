use viewer_core::xml::parse_document;
use viewer_core::ViewerError;

#[test]
fn namespace_handling_uses_local_name() {
    let root = parse_document(
        r#"<w:document xmlns:w="urn:test"><w:body><w:p id="1" /></w:body></w:document>"#,
    )
    .expect("xml should parse");

    assert_eq!(root.local_name(), "document");
    assert_eq!(root.child("body").expect("body should exist").local_name(), "body");
}

#[test]
fn missing_attribute_returns_invalid_document() {
    let root = parse_document(r#"<w:p xmlns:w="urn:test" />"#).expect("xml should parse");

    let error = root.required_attribute("id").expect_err("missing id should fail");
    assert!(matches!(error, ViewerError::InvalidDocument));
}

#[test]
fn malformed_xml_returns_invalid_document() {
    let error = parse_document("<w:document><w:body>").expect_err("malformed xml should fail");

    assert!(matches!(error, ViewerError::InvalidDocument));
}

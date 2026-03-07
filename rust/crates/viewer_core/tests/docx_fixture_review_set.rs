use std::path::PathBuf;

#[test]
fn docx_review_fixture_set_is_present() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root");
    let required = [
        "fixtures/docx/README.md",
        "fixtures/docx/docx_plain_text.docx",
        "fixtures/docx/docx_styles_lists.docx",
        "fixtures/docx/docx_tables_images.docx",
        "fixtures/docx/docx_multi_section.docx",
        "fixtures/encrypted/docx_password_stub.docx",
        "fixtures/regression/docx_acceptance_plain_text.docx",
    ];

    for path in required {
        let absolute_path = workspace_root.join(path);
        assert!(
            absolute_path.exists(),
            "required review fixture missing: {path}"
        );
    }
}

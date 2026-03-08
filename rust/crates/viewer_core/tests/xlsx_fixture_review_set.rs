use std::path::PathBuf;

#[test]
fn xlsx_review_fixture_set_is_present() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root");
    let required = [
        "fixtures/xlsx/README.md",
        "fixtures/xlsx/xlsx_basic_grid.xlsx",
        "fixtures/xlsx/xlsx_merges_frozen_formulas.xlsx",
    ];

    for path in required {
        let absolute_path = workspace_root.join(path);
        assert!(
            absolute_path.exists(),
            "required review fixture missing: {path}"
        );
    }
}

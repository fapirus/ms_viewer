use std::path::PathBuf;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("project root should resolve")
}

#[test]
fn required_fixture_directories_exist() {
    let root = project_root();
    let fixtures = root.join("fixtures");

    for path in [
        fixtures.join("docx"),
        fixtures.join("pptx"),
        fixtures.join("xlsx"),
        fixtures.join("encrypted"),
        fixtures.join("regression"),
    ] {
        assert!(path.exists(), "missing fixture path: {}", path.display());
        assert!(
            path.is_dir(),
            "fixture path should be directory: {}",
            path.display()
        );
    }
}

#[test]
fn fixture_readme_exists() {
    let root = project_root();
    let readme = root.join("fixtures/README.md");

    assert!(readme.exists(), "fixtures README should exist");
}

use viewer_core::text::{resolve_font_family, Platform, Script};

#[test]
fn platform_specific_mapping_wins() {
    assert_eq!(
        resolve_font_family(Platform::Windows, "Calibri", Script::Latin),
        "Arial"
    );
    assert_eq!(
        resolve_font_family(Platform::Android, "Cambria", Script::Latin),
        "Noto Serif"
    );
}

#[test]
fn cjk_generic_fallback_is_used_for_unknown_fonts() {
    assert_eq!(
        resolve_font_family(Platform::MacOs, "Unknown Font", Script::Cjk),
        "Apple SD Gothic Neo"
    );
}

#[test]
fn latin_generic_fallback_is_used_for_unknown_fonts() {
    assert_eq!(
        resolve_font_family(Platform::Ios, "Unknown Font", Script::Latin),
        "Helvetica"
    );
}

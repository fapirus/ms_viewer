#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOs,
    Android,
    Ios,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Latin,
    Cjk,
}

pub fn resolve_font_family(platform: Platform, requested: &str, script: Script) -> &'static str {
    match (platform, requested) {
        (Platform::Windows, "Calibri") => "Arial",
        (Platform::Windows, "Cambria") => "Times New Roman",
        (Platform::Windows, "Malgun Gothic") => "Malgun Gothic",
        (Platform::Windows, "Batang") => "Malgun Gothic",
        (Platform::MacOs, "Calibri") => "Helvetica",
        (Platform::MacOs, "Cambria") => "Times",
        (Platform::MacOs, "Malgun Gothic") => "Apple SD Gothic Neo",
        (Platform::MacOs, "Batang") => "Apple SD Gothic Neo",
        (Platform::Android, "Calibri") => "Roboto",
        (Platform::Android, "Cambria") => "Noto Serif",
        (Platform::Android, "Malgun Gothic") => "Noto Sans CJK KR",
        (Platform::Android, "Batang") => "Noto Sans CJK KR",
        (Platform::Ios, "Calibri") => "Helvetica",
        (Platform::Ios, "Cambria") => "Times New Roman",
        (Platform::Ios, "Malgun Gothic") => "Apple SD Gothic Neo",
        (Platform::Ios, "Batang") => "Apple SD Gothic Neo",
        _ => generic_fallback(platform, script),
    }
}

fn generic_fallback(platform: Platform, script: Script) -> &'static str {
    match (platform, script) {
        (Platform::Windows, Script::Latin) => "Arial",
        (Platform::Windows, Script::Cjk) => "Malgun Gothic",
        (Platform::MacOs, Script::Latin) => "Helvetica",
        (Platform::MacOs, Script::Cjk) => "Apple SD Gothic Neo",
        (Platform::Android, Script::Latin) => "Roboto",
        (Platform::Android, Script::Cjk) => "Noto Sans CJK KR",
        (Platform::Ios, Script::Latin) => "Helvetica",
        (Platform::Ios, Script::Cjk) => "Apple SD Gothic Neo",
    }
}

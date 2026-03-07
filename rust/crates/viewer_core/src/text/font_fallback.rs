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
        (Platform::Windows, "Aptos") => "Arial",
        (Platform::Windows, "Cambria") => "Times New Roman",
        (Platform::Windows, "Malgun Gothic") => "Malgun Gothic",
        (Platform::Windows, "맑은 고딕") => "Malgun Gothic",
        (Platform::Windows, "Batang") => "Malgun Gothic",
        (Platform::Windows, "바탕") => "Malgun Gothic",
        (Platform::Windows, "Dotum") => "Malgun Gothic",
        (Platform::Windows, "돋움") => "Malgun Gothic",
        (Platform::Windows, "Gulim") => "Malgun Gothic",
        (Platform::Windows, "굴림") => "Malgun Gothic",
        (Platform::Windows, "Nanum Gothic") => "Malgun Gothic",
        (Platform::MacOs, "Calibri") => "Helvetica",
        (Platform::MacOs, "Aptos") => "Helvetica",
        (Platform::MacOs, "Cambria") => "Times",
        (Platform::MacOs, "Malgun Gothic") => "Apple SD Gothic Neo",
        (Platform::MacOs, "맑은 고딕") => "Apple SD Gothic Neo",
        (Platform::MacOs, "Batang") => "Apple SD Gothic Neo",
        (Platform::MacOs, "바탕") => "Apple SD Gothic Neo",
        (Platform::MacOs, "Dotum") => "Apple SD Gothic Neo",
        (Platform::MacOs, "돋움") => "Apple SD Gothic Neo",
        (Platform::MacOs, "Gulim") => "Apple SD Gothic Neo",
        (Platform::MacOs, "굴림") => "Apple SD Gothic Neo",
        (Platform::MacOs, "Nanum Gothic") => "Apple SD Gothic Neo",
        (Platform::Android, "Calibri") => "Roboto",
        (Platform::Android, "Aptos") => "Roboto",
        (Platform::Android, "Cambria") => "Noto Serif",
        (Platform::Android, "Malgun Gothic") => "Noto Sans CJK KR",
        (Platform::Android, "맑은 고딕") => "Noto Sans CJK KR",
        (Platform::Android, "Batang") => "Noto Sans CJK KR",
        (Platform::Android, "바탕") => "Noto Sans CJK KR",
        (Platform::Android, "Dotum") => "Noto Sans CJK KR",
        (Platform::Android, "돋움") => "Noto Sans CJK KR",
        (Platform::Android, "Gulim") => "Noto Sans CJK KR",
        (Platform::Android, "굴림") => "Noto Sans CJK KR",
        (Platform::Android, "Nanum Gothic") => "Noto Sans CJK KR",
        (Platform::Ios, "Calibri") => "Helvetica",
        (Platform::Ios, "Aptos") => "Helvetica",
        (Platform::Ios, "Cambria") => "Times New Roman",
        (Platform::Ios, "Malgun Gothic") => "Apple SD Gothic Neo",
        (Platform::Ios, "맑은 고딕") => "Apple SD Gothic Neo",
        (Platform::Ios, "Batang") => "Apple SD Gothic Neo",
        (Platform::Ios, "바탕") => "Apple SD Gothic Neo",
        (Platform::Ios, "Dotum") => "Apple SD Gothic Neo",
        (Platform::Ios, "돋움") => "Apple SD Gothic Neo",
        (Platform::Ios, "Gulim") => "Apple SD Gothic Neo",
        (Platform::Ios, "굴림") => "Apple SD Gothic Neo",
        (Platform::Ios, "Nanum Gothic") => "Apple SD Gothic Neo",
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

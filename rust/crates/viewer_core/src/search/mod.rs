use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub page_index: u32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SheetCellMatch {
    pub row: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchMatch {
    pub query: String,
    pub page_index: u32,
    pub start: usize,
    pub end: usize,
    pub preview: String,
    #[serde(default)]
    pub sheet_cell: Option<SheetCellMatch>,
}

pub fn search_pages(pages: &[SearchPage], query: &str) -> Vec<SearchMatch> {
    let trimmed_query = query.trim();
    if trimmed_query.is_empty() {
        return Vec::new();
    }

    let normalized_query = trimmed_query.to_lowercase();
    let mut matches = Vec::new();

    for page in pages {
        let normalized_text = page.text.to_lowercase();
        let mut offset = 0;

        while let Some(found) = normalized_text[offset..].find(&normalized_query) {
            let start = offset + found;
            let end = start + normalized_query.len();
            matches.push(SearchMatch {
                query: trimmed_query.to_string(),
                page_index: page.page_index,
                start,
                end,
                preview: build_preview(&page.text, start, end),
                sheet_cell: None,
            });
            offset = end;
        }
    }

    matches
}

fn build_preview(text: &str, start: usize, end: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let prefix_start = start.saturating_sub(20);
    let suffix_end = (end + 20).min(chars.len());
    chars[prefix_start..suffix_end]
        .iter()
        .collect::<String>()
        .replace('\n', " ")
        .trim()
        .to_string()
}

use std::env;
use std::process::ExitCode;

use viewer_ffi::{
    get_page_render_model_json, get_selection_page_json, open_document_json,
    search_document_pages_json,
};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!("usage: viewer_cli <command> <request_json>");
        return ExitCode::from(2);
    };
    let Some(request_json) = args.next() else {
        eprintln!("missing request_json argument");
        return ExitCode::from(2);
    };

    let response = match command.as_str() {
        "open-document" => serde_json::to_string(&open_document_json(&request_json)),
        "get-page-render-model" => {
            serde_json::to_string(&get_page_render_model_json(&request_json))
        }
        "search-document" => serde_json::to_string(&search_document_pages_json(&request_json)),
        "get-selection-page" => serde_json::to_string(&get_selection_page_json(&request_json)),
        other => {
            eprintln!("unknown command: {other}");
            return ExitCode::from(2);
        }
    };

    match response {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("failed to serialize response: {error}");
            ExitCode::from(1)
        }
    }
}

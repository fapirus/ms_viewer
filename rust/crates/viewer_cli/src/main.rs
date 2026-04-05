use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::ExitCode;

use base64::Engine;
use format_xlsx::{
    build_sheet_render_model_from_package, build_visible_window_render_model_from_package,
    parse_xlsx_package, search_workbook_from_package, ParsedXlsxPackage, XlsxVisibleWindow,
};
use serde::{Deserialize, Serialize};
use viewer_core::archive::OoxmlArchive;
use viewer_core::model::PageRenderModel;
use viewer_core::ViewerError;
use viewer_ffi::{
    get_page_render_model, get_page_render_model_json, get_selection_page, get_selection_page_json,
    open_document, open_document_json, search_document_pages, search_document_pages_json,
    DocumentSource, GetPageRenderModelRequest, GetPageRenderModelResponse, GetSelectionPageRequest,
    GetSelectionPageResponse, OpenDocumentRequest, OpenDocumentResponse, SearchDocumentRequest,
    SearchDocumentResponse,
};

#[derive(Debug, Clone)]
struct CachedXlsxDocument {
    package: ParsedXlsxPackage,
}

#[derive(Default)]
struct CliServerState {
    xlsx_documents: HashMap<String, CachedXlsxDocument>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServeEnvelope {
    command: String,
    request_json: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ServeResponse {
    Open(OpenDocumentResponse),
    Page(GetPageRenderModelResponse),
    Search(SearchDocumentResponse),
    Selection(GetSelectionPageResponse),
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!("usage: viewer_cli <command> <request_json>");
        return ExitCode::from(2);
    };

    if command == "serve" {
        return serve();
    }

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

fn serve() -> ExitCode {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut state = CliServerState::default();

    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            eprintln!("failed to read stdin");
            return ExitCode::from(1);
        };
        if line.trim().is_empty() {
            continue;
        }

        let response = match handle_serve_line(&line, &mut state) {
            Ok(response) => response,
            Err(error) => {
                eprintln!("failed to handle serve request: {error}");
                return ExitCode::from(1);
            }
        };

        if writeln!(stdout, "{response}").is_err() || stdout.flush().is_err() {
            eprintln!("failed to write serve response");
            return ExitCode::from(1);
        }
    }

    ExitCode::SUCCESS
}

fn handle_serve_line(line: &str, state: &mut CliServerState) -> Result<String, serde_json::Error> {
    let envelope: ServeEnvelope = serde_json::from_str(line)?;
    let response = match envelope.command.as_str() {
        "open-document" => ServeResponse::Open(handle_open_document(
            &envelope.request_json,
            state,
        )),
        "get-page-render-model" => ServeResponse::Page(handle_get_page_render_model(
            &envelope.request_json,
            state,
        )),
        "search-document" => {
            ServeResponse::Search(handle_search_document(&envelope.request_json, state))
        }
        "get-selection-page" => {
            ServeResponse::Selection(handle_get_selection_page(&envelope.request_json, state))
        }
        other => {
            eprintln!("unknown serve command: {other}");
            ServeResponse::Page(GetPageRenderModelResponse::Error(
                ViewerError::InvalidDocument.to_error_response(),
            ))
        }
    };

    serde_json::to_string(&response)
}

fn handle_open_document(
    request_json: &str,
    state: &mut CliServerState,
) -> OpenDocumentResponse {
    let request: OpenDocumentRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(_) => return OpenDocumentResponse::Error(ViewerError::InvalidDocument.to_error_response()),
    };

    let response = open_document(request.clone());
    if let OpenDocumentResponse::Success(success) = &response {
        if matches!(success.kind, viewer_core::model::DocumentKind::Xlsx) {
            if let Ok(archive) = open_archive_from_source(&request.source) {
                if let Ok(package) = parse_xlsx_package(&archive) {
                    state.xlsx_documents.insert(
                        success.document_id.clone(),
                        CachedXlsxDocument { package },
                    );
                }
            }
        }
    }

    response
}

fn handle_get_page_render_model(
    request_json: &str,
    state: &CliServerState,
) -> GetPageRenderModelResponse {
    let request: GetPageRenderModelRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(_) => {
            return GetPageRenderModelResponse::Error(
                ViewerError::InvalidDocument.to_error_response(),
            )
        }
    };

    if let Some(document) = state.xlsx_documents.get(&request.document_id) {
        return render_cached_xlsx_page(&request, document);
    }

    get_page_render_model(request)
}

fn handle_search_document(
    request_json: &str,
    state: &CliServerState,
) -> SearchDocumentResponse {
    let request: SearchDocumentRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(_) => {
            return SearchDocumentResponse::Error(
                ViewerError::InvalidDocument.to_error_response(),
            )
        }
    };

    if let Some(document) = state.xlsx_documents.get(&request.document_id) {
        return match search_workbook_from_package(&document.package, &request.query) {
            Ok(matches) => SearchDocumentResponse::Success(matches),
            Err(error) => SearchDocumentResponse::Error(error.to_error_response()),
        };
    }

    search_document_pages(request)
}

fn handle_get_selection_page(
    request_json: &str,
    state: &CliServerState,
) -> GetSelectionPageResponse {
    let request: GetSelectionPageRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(_) => {
            return GetSelectionPageResponse::Error(
                ViewerError::InvalidDocument.to_error_response(),
            )
        }
    };

    if let Some(document) = state.xlsx_documents.get(&request.document_id) {
        return match build_sheet_render_model_from_package(
            &document.package,
            request.page_index as usize,
        ) {
            Ok(page) => GetSelectionPageResponse::Success(page),
            Err(error) => GetSelectionPageResponse::Error(error.to_error_response()),
        };
    }

    get_selection_page(request)
}

fn render_cached_xlsx_page(
    request: &GetPageRenderModelRequest,
    document: &CachedXlsxDocument,
) -> GetPageRenderModelResponse {
    let result: Result<PageRenderModel, ViewerError> = if let Some(window) = request.sheet_window.as_ref()
    {
        if window.end_row < window.start_row || window.end_column < window.start_column {
            Err(ViewerError::InvalidDocument)
        } else {
            build_visible_window_render_model_from_package(
                &document.package,
                request.page_index as usize,
                &XlsxVisibleWindow {
                    start_row: window.start_row,
                    start_column: window.start_column,
                    row_count: window.end_row - window.start_row + 1,
                    column_count: window.end_column - window.start_column + 1,
                },
            )
        }
    } else {
        build_sheet_render_model_from_package(&document.package, request.page_index as usize)
    };

    match result {
        Ok(page) => GetPageRenderModelResponse::Success(page),
        Err(error) => GetPageRenderModelResponse::Error(error.to_error_response()),
    }
}

fn open_archive_from_source(source: &DocumentSource) -> Result<OoxmlArchive, ViewerError> {
    match source {
        DocumentSource::Path(path) => OoxmlArchive::open_path(Path::new(path)),
        DocumentSource::BytesBase64(value) => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(value)
                .map_err(|_| ViewerError::InvalidDocument)?;
            OoxmlArchive::open_bytes(bytes)
        }
    }
}

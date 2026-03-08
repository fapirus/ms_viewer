import 'render_model.dart';

class OpenOptions {
  const OpenOptions({
    this.password,
    this.preferLazyLoading = true,
  });

  final String? password;
  final bool preferLazyLoading;

  Map<String, Object?> toJson() {
    return {
      'password': password,
      'preferLazyLoading': preferLazyLoading,
    };
  }
}

enum DocumentSourceKind { path, bytesBase64 }

class OpenDocumentSource {
  const OpenDocumentSource._({
    required this.kind,
    required this.value,
  });

  const OpenDocumentSource.path(String value)
    : this._(kind: DocumentSourceKind.path, value: value);

  const OpenDocumentSource.bytesBase64(String value)
    : this._(kind: DocumentSourceKind.bytesBase64, value: value);

  final DocumentSourceKind kind;
  final String value;

  Map<String, Object?> toJson() {
    return {
      'kind': kind.name,
      'value': value,
    };
  }
}

class OpenDocumentRequest {
  const OpenDocumentRequest({
    required this.source,
    this.options = const OpenOptions(),
  });

  final OpenDocumentSource source;
  final OpenOptions options;

  Map<String, Object?> toJson() {
    return {
      'source': source.toJson(),
      'options': options.toJson(),
    };
  }
}

class GetPageRenderModelRequest {
  const GetPageRenderModelRequest({
    required this.source,
    required this.documentId,
    required this.pageIndex,
    this.sheetWindow,
    this.options = const OpenOptions(),
  });

  final OpenDocumentSource source;
  final String documentId;
  final int pageIndex;
  final SheetWindow? sheetWindow;
  final OpenOptions options;

  Map<String, Object?> toJson() {
    return {
      'source': source.toJson(),
      'documentId': documentId,
      'pageIndex': pageIndex,
      if (sheetWindow != null) 'sheetWindow': sheetWindow!.toJson(),
      'options': options.toJson(),
    };
  }
}

class SheetWindow {
  const SheetWindow({
    required this.startRow,
    required this.endRow,
    required this.startColumn,
    required this.endColumn,
  });

  final int startRow;
  final int endRow;
  final int startColumn;
  final int endColumn;

  Map<String, Object?> toJson() {
    return {
      'startRow': startRow,
      'endRow': endRow,
      'startColumn': startColumn,
      'endColumn': endColumn,
    };
  }
}

class SearchDocumentRequest {
  const SearchDocumentRequest({
    required this.source,
    required this.documentId,
    required this.query,
    this.options = const OpenOptions(),
  });

  final OpenDocumentSource source;
  final String documentId;
  final String query;
  final OpenOptions options;

  Map<String, Object?> toJson() {
    return {
      'source': source.toJson(),
      'documentId': documentId,
      'query': query,
      'options': options.toJson(),
    };
  }
}

class GetSelectionPageRequest {
  const GetSelectionPageRequest({
    required this.source,
    required this.documentId,
    required this.pageIndex,
    this.options = const OpenOptions(),
  });

  final OpenDocumentSource source;
  final String documentId;
  final int pageIndex;
  final OpenOptions options;

  Map<String, Object?> toJson() {
    return {
      'source': source.toJson(),
      'documentId': documentId,
      'pageIndex': pageIndex,
      'options': options.toJson(),
    };
  }
}

enum ViewerErrorCode {
  unsupportedFormat,
  passwordRequired,
  invalidPassword,
  unsupportedEncryption,
  ioError,
  invalidDocument,
  notImplemented,
}

enum DocumentKind { docx, pptx, xlsx }

class DocumentCapabilities {
  const DocumentCapabilities({
    required this.search,
    required this.textSelection,
    required this.passwordProtected,
  });

  final bool search;
  final bool textSelection;
  final bool passwordProtected;

  factory DocumentCapabilities.fromJson(Map<String, Object?> json) {
    return DocumentCapabilities(
      search: json['search'] as bool? ?? false,
      textSelection: json['textSelection'] as bool? ?? false,
      passwordProtected: json['passwordProtected'] as bool? ?? false,
    );
  }
}

class OpenDocumentSuccess {
  const OpenDocumentSuccess({
    required this.documentId,
    required this.kind,
    required this.title,
    required this.pageCount,
    required this.capabilities,
  });

  final String documentId;
  final DocumentKind kind;
  final String title;
  final int pageCount;
  final DocumentCapabilities capabilities;

  factory OpenDocumentSuccess.fromJson(Map<String, Object?> json) {
    return OpenDocumentSuccess(
      documentId: json['documentId'] as String,
      kind: DocumentKind.values.byName(json['kind'] as String),
      title: json['title'] as String,
      pageCount: json['pageCount'] as int,
      capabilities: DocumentCapabilities.fromJson(
        json['capabilities'] as Map<String, Object?>,
      ),
    );
  }
}

class OpenDocumentError {
  const OpenDocumentError({required this.code, required this.message});

  final ViewerErrorCode code;
  final String message;

  factory OpenDocumentError.fromJson(Map<String, Object?> json) {
    return OpenDocumentError(
      code: _viewerErrorCodeFromWire(json['code'] as String),
      message: json['message'] as String,
    );
  }
}

sealed class OpenDocumentResult {
  const OpenDocumentResult();

  factory OpenDocumentResult.fromJson(Map<String, Object?> json) {
    if (json.containsKey('code')) {
      return OpenDocumentFailure(OpenDocumentError.fromJson(json));
    }

    return OpenDocumentOpened(OpenDocumentSuccess.fromJson(json));
  }
}

class OpenDocumentOpened extends OpenDocumentResult {
  const OpenDocumentOpened(this.document);

  final OpenDocumentSuccess document;
}

class OpenDocumentFailure extends OpenDocumentResult {
  const OpenDocumentFailure(this.error);

  final OpenDocumentError error;
}

sealed class GetPageRenderModelResult {
  const GetPageRenderModelResult();

  factory GetPageRenderModelResult.fromJson(Map<String, Object?> json) {
    if (json.containsKey('code')) {
      return GetPageRenderModelFailure(OpenDocumentError.fromJson(json));
    }

    return GetPageRenderModelSuccess(
      PageRenderModel.fromJson(json),
    );
  }
}

class GetPageRenderModelSuccess extends GetPageRenderModelResult {
  const GetPageRenderModelSuccess(this.page);

  final PageRenderModel page;
}

class GetPageRenderModelFailure extends GetPageRenderModelResult {
  const GetPageRenderModelFailure(this.error);

  final OpenDocumentError error;
}

class SearchMatchModel {
  const SearchMatchModel({
    required this.query,
    required this.pageIndex,
    required this.start,
    required this.end,
    required this.preview,
  });

  final String query;
  final int pageIndex;
  final int start;
  final int end;
  final String preview;

  factory SearchMatchModel.fromJson(Map<String, Object?> json) {
    return SearchMatchModel(
      query: json['query'] as String,
      pageIndex: json['pageIndex'] as int,
      start: json['start'] as int,
      end: json['end'] as int,
      preview: json['preview'] as String,
    );
  }
}

sealed class SearchDocumentResult {
  const SearchDocumentResult();

  factory SearchDocumentResult.fromJson(Object? json) {
    if (json is Map<String, Object?> && json.containsKey('code')) {
      return SearchDocumentFailure(OpenDocumentError.fromJson(json));
    }
    if (json is List<Object?>) {
      return SearchDocumentSuccess(
        json
            .cast<Map<String, Object?>>()
            .map(SearchMatchModel.fromJson)
            .toList(),
      );
    }

    throw ArgumentError.value(json, 'json', 'Invalid search result payload');
  }
}

class SearchDocumentSuccess extends SearchDocumentResult {
  const SearchDocumentSuccess(this.matches);

  final List<SearchMatchModel> matches;
}

class SearchDocumentFailure extends SearchDocumentResult {
  const SearchDocumentFailure(this.error);

  final OpenDocumentError error;
}

sealed class GetSelectionPageResult {
  const GetSelectionPageResult();

  factory GetSelectionPageResult.fromJson(Map<String, Object?> json) {
    if (json.containsKey('code')) {
      return GetSelectionPageFailure(OpenDocumentError.fromJson(json));
    }

    return GetSelectionPageSuccess(PageRenderModel.fromJson(json));
  }
}

class GetSelectionPageSuccess extends GetSelectionPageResult {
  const GetSelectionPageSuccess(this.page);

  final PageRenderModel page;
}

class GetSelectionPageFailure extends GetSelectionPageResult {
  const GetSelectionPageFailure(this.error);

  final OpenDocumentError error;
}

ViewerErrorCode _viewerErrorCodeFromWire(String value) {
  return switch (value) {
    'unsupported_format' || 'unsupportedFormat' =>
      ViewerErrorCode.unsupportedFormat,
    'password_required' || 'passwordRequired' =>
      ViewerErrorCode.passwordRequired,
    'invalid_password' || 'invalidPassword' =>
      ViewerErrorCode.invalidPassword,
    'unsupported_encryption' || 'unsupportedEncryption' =>
      ViewerErrorCode.unsupportedEncryption,
    'io_error' || 'ioError' => ViewerErrorCode.ioError,
    'invalid_document' || 'invalidDocument' =>
      ViewerErrorCode.invalidDocument,
    'not_implemented' || 'notImplemented' =>
      ViewerErrorCode.notImplemented,
    _ => throw ArgumentError.value(value, 'value', 'Unknown viewer error code'),
  };
}

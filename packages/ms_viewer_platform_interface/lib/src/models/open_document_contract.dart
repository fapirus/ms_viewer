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

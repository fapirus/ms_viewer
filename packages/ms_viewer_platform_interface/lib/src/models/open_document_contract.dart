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

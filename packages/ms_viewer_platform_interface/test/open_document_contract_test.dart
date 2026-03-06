import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

void main() {
  test('open document request encodes to rust-compatible json-like map', () {
    const request = OpenDocumentRequest(
      source: OpenDocumentSource.path('/tmp/sample.docx'),
      options: OpenOptions(password: 'secret', preferLazyLoading: false),
    );

    expect(request.toJson(), {
      'source': {'kind': 'path', 'value': '/tmp/sample.docx'},
      'options': {'password': 'secret', 'preferLazyLoading': false},
    });
  });

  test('open document success decodes from json-like map', () {
    final success = OpenDocumentSuccess.fromJson({
      'documentId': 'doc_001',
      'kind': 'docx',
      'title': 'sample.docx',
      'pageCount': 12,
      'capabilities': {
        'search': true,
        'textSelection': true,
        'passwordProtected': false,
      },
    });

    expect(success.documentId, 'doc_001');
    expect(success.kind, DocumentKind.docx);
    expect(success.capabilities.textSelection, isTrue);
  });

  test('open document error decodes from json-like map', () {
    final error = OpenDocumentError.fromJson({
      'code': 'password_required',
      'message': 'Password is required to open this document.',
    });

    expect(error.code, ViewerErrorCode.passwordRequired);
    expect(error.message, contains('Password'));
  });

  test('open document error also accepts legacy camelCase code', () {
    final error = OpenDocumentError.fromJson({
      'code': 'invalidPassword',
      'message': 'Password is invalid.',
    });

    expect(error.code, ViewerErrorCode.invalidPassword);
  });

  test('open document result decodes success response', () {
    final result = OpenDocumentResult.fromJson({
      'documentId': 'doc_001',
      'kind': 'docx',
      'title': 'sample.docx',
      'pageCount': 0,
      'capabilities': {
        'search': false,
        'textSelection': false,
        'passwordProtected': false,
      },
    });

    expect(result, isA<OpenDocumentOpened>());
  });

  test('open document result decodes error response', () {
    final result = OpenDocumentResult.fromJson({
      'code': 'unsupported_format',
      'message': 'Unsupported format.',
    });

    expect(result, isA<OpenDocumentFailure>());
  });
}

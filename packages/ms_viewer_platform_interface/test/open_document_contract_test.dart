import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

void main() {
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
      'code': 'passwordRequired',
      'message': 'Password is required to open this document.',
    });

    expect(error.code, ViewerErrorCode.passwordRequired);
    expect(error.message, contains('Password'));
  });
}

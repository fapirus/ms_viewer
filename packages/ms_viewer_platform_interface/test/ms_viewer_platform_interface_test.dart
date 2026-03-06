import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

void main() {
  test('default platform exists', () {
    expect(MsViewerPlatform.instance, isNotNull);
  });

  test('default platform returns not implemented open result', () async {
    final result = await MsViewerPlatform.instance.openDocument(
      const OpenDocumentRequest(
        source: OpenDocumentSource.path('/tmp/sample.docx'),
      ),
    );

    expect(result, isA<OpenDocumentFailure>());
    expect((result as OpenDocumentFailure).error.code, ViewerErrorCode.notImplemented);
  });
}

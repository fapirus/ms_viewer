import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';

void main() {
  test('controller attaches document', () {
    final controller = MsViewerController();
    const document = DocumentDescriptor(
      id: 'doc-1',
      kind: DocumentKind.docx,
      title: 'Sample',
      pageCount: 1,
    );

    controller.attachDocument(document);

    expect(controller.document, document);
  });
}

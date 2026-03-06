import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';

void main() {
  test('docx viewport request stores page index', () {
    const request = ViewportRequest.docx(index: 3);

    expect(request.kind, DocumentKind.docx);
    expect(request.index, 3);
    expect(request.sheetWindow, isNull);
  });

  test('xlsx viewport request carries visible window', () {
    const request = ViewportRequest.xlsx(
      index: 0,
      sheetWindow: SheetWindow(
        startRow: 0,
        endRow: 40,
        startColumn: 0,
        endColumn: 8,
      ),
    );

    expect(request.kind, DocumentKind.xlsx);
    expect(request.sheetWindow?.endRow, 40);
  });
}

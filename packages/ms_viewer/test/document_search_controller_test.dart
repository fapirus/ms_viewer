import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';

void main() {
  test('next/previous navigation state test', () {
    final controller = DocumentSearchController();

    controller.updateResults(
      query: 'alpha',
      results: const [
        SearchResult(
          query: 'alpha',
          pageIndex: 0,
          start: 0,
          end: 5,
          preview: 'first',
          sheetCell: null,
        ),
        SearchResult(
          query: 'alpha',
          pageIndex: 1,
          start: 6,
          end: 11,
          preview: 'second',
          sheetCell: null,
        ),
        SearchResult(
          query: 'alpha',
          pageIndex: 2,
          start: 12,
          end: 17,
          preview: 'third',
          sheetCell: null,
        ),
      ],
    );

    expect(controller.currentIndex, 0);
    expect(controller.currentResult!.preview, 'first');

    controller.next();
    expect(controller.currentIndex, 1);
    expect(controller.currentResult!.preview, 'second');

    controller.next();
    expect(controller.currentIndex, 2);

    controller.next();
    expect(controller.currentIndex, 0);

    controller.previous();
    expect(controller.currentIndex, 2);
    expect(controller.currentResult!.pageIndex, 2);
  });
}

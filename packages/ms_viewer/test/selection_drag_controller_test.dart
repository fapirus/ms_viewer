import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';

void main() {
  test('drag selection state test', () {
    final controller = SelectionDragController();

    controller.start(const Offset(12, 24));
    expect(controller.isDragging, isTrue);
    expect(controller.selectionRect, const Rect.fromLTWH(12, 24, 0, 0));

    controller.update(const Offset(40, 52));
    expect(controller.selectionRect, const Rect.fromLTRB(12, 24, 40, 52));

    controller.clear();
    expect(controller.isDragging, isFalse);
    expect(controller.selectionRect, isNull);
  });
}

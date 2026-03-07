import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';

void main() {
  testWidgets('selection highlight widget test', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: SizedBox(
            width: 200,
            height: 200,
            child: SelectionHighlightOverlay(
              highlights: [
                Rect.fromLTWH(10, 12, 80, 18),
                Rect.fromLTWH(10, 36, 64, 18),
              ],
            ),
          ),
        ),
      ),
    );

    final positioned = tester.widgetList<Positioned>(find.byType(Positioned)).toList();
    expect(positioned, hasLength(2));
    expect(positioned[0].left, 10);
    expect(positioned[0].top, 12);
    expect(positioned[1].top, 36);
  });
}

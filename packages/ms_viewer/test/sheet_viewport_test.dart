import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

void main() {
  PageRenderModel buildSheetPage({int columns = 2, int rows = 2}) {
    final nodes = <Map<String, Object?>>[];
    for (var row = 0; row < rows; row++) {
      for (var column = 0; column < columns; column++) {
        nodes.add({
          'type': 'box',
          'bounds': {
            'x': column * 140.0,
            'y': row * 40.0,
            'width': 140.0,
            'height': 40.0,
          },
          'fillColorHex': row == 0 ? '#1F4E78' : '#FFFFFF',
          'strokeColorHex': '#D0D7DE',
          'strokeWidth': 1.0,
        });
      }
    }

    nodes.add({
      'type': 'text',
      'text': 'Budget',
      'bounds': {'x': 20.0, 'y': 10.0, 'width': 80.0, 'height': 18.0},
      'style': {
        'fontFamily': 'Calibri',
        'fontSize': 12.0,
        'bold': true,
        'italic': false,
        'colorHex': '#FFFFFF',
      },
      'range': {'start': 0, 'end': 6},
    });

    return PageRenderModel.fromJson({
      'pageIndex': 0,
      'width': columns * 140.0,
      'height': rows * 40.0,
      'nodes': nodes,
      'selectionAnchors': [
        {'nodeIndex': nodes.length - 1, 'charIndex': 0, 'x': 20.0, 'y': 10.0},
        {'nodeIndex': nodes.length - 1, 'charIndex': 6, 'x': 72.0, 'y': 10.0},
      ],
    });
  }

  testWidgets('sheet viewport renders pinned headers and body canvas', (
    tester,
  ) async {
    final page = buildSheetPage();

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Padding(
            padding: const EdgeInsets.all(24),
            child: SheetViewport(page: page),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byType(SheetViewport), findsOneWidget);
    expect(find.byKey(const ValueKey('sheet-corner-cell')), findsOneWidget);
    expect(find.byKey(const ValueKey('sheet-column-header-1')), findsOneWidget);
    expect(find.byKey(const ValueKey('sheet-column-header-2')), findsOneWidget);
    expect(find.byKey(const ValueKey('sheet-row-header-1')), findsOneWidget);
    expect(find.byKey(const ValueKey('sheet-body-canvas')), findsOneWidget);
  });

  testWidgets('sheet viewport keeps headers pinned while body scrolls in 2D', (
    tester,
  ) async {
    final page = buildSheetPage(columns: 6, rows: 12);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 320,
              height: 240,
              child: SheetViewport(page: page),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    final columnHeader = find.byKey(const ValueKey('sheet-column-header-4'));
    final rowHeader = find.byKey(const ValueKey('sheet-row-header-8'));
    final bodyViewport = find.byKey(const ValueKey('sheet-body-viewport'));

    final initialColumnLeft = tester.getTopLeft(columnHeader).dx;
    final initialRowTop = tester.getTopLeft(rowHeader).dy;

    await tester.drag(bodyViewport, const Offset(-180, 0), warnIfMissed: false);
    await tester.pumpAndSettle();

    expect(tester.getTopLeft(columnHeader).dx, lessThan(initialColumnLeft));

    await tester.drag(bodyViewport, const Offset(0, -120), warnIfMissed: false);
    await tester.pumpAndSettle();

    expect(tester.getTopLeft(rowHeader).dy, lessThan(initialRowTop));
    expect(find.byKey(const ValueKey('sheet-corner-cell')), findsOneWidget);
  });
}

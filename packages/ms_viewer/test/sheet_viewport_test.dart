import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;

void main() {
  platform.PageRenderModel buildSheetPage({
    int columns = 2,
    int rows = 2,
    bool includeFrozenPane = false,
    int startRow = 1,
    int startColumn = 1,
    int effectiveEndRow = 40,
    int effectiveEndColumn = 16,
    double cellWidth = 140,
    double cellHeight = 40,
  }) {
    final nodes = <Map<String, Object?>>[];
    for (var row = 0; row < rows; row++) {
      for (var column = 0; column < columns; column++) {
        nodes.add({
          'type': 'box',
          'bounds': {
            'x': column * cellWidth,
            'y': row * cellHeight,
            'width': cellWidth,
            'height': cellHeight,
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
        'underline': false,
        'colorHex': '#FFFFFF',
      },
      'range': {'start': 0, 'end': 6},
    });

    return platform.PageRenderModel.fromJson({
      'pageIndex': 0,
      'width': columns * cellWidth,
      'height': rows * cellHeight,
      'nodes': nodes,
      'selectionAnchors': [
        {'nodeIndex': nodes.length - 1, 'charIndex': 0, 'x': 20.0, 'y': 10.0},
        {'nodeIndex': nodes.length - 1, 'charIndex': 6, 'x': 72.0, 'y': 10.0},
      ],
      'sheetCells': [
        for (var row = 0; row < rows; row++)
          for (var column = 0; column < columns; column++)
            {
              'row': startRow + row,
              'column': startColumn + column,
              'bounds': {
                'x': column * cellWidth,
                'y': row * cellHeight,
                'width': cellWidth,
                'height': cellHeight,
              },
            },
      ],
      'sheetViewport': {
        'window': {
          'startRow': startRow,
          'endRow': startRow + rows - 1,
          'startColumn': startColumn,
          'endColumn': startColumn + columns - 1,
        },
        'effectiveBounds': {
          'startRow': 1,
          'endRow': effectiveEndRow,
          'startColumn': 1,
          'endColumn': effectiveEndColumn,
        },
        if (includeFrozenPane)
          'frozenPane': {
            'frozenRows': 1,
            'frozenColumns': 1,
            'topLeftCell': 'B2',
          },
        'visibleRows': [for (var row = 0; row < rows; row++) startRow + row],
        'visibleColumns': [
          for (var column = 0; column < columns; column++) startColumn + column,
        ],
      },
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

  testWidgets(
    'sheet viewport derives headers from sheet cell metadata when sparse nodes are used',
    (tester) async {
      final page = platform.PageRenderModel.fromJson({
        'pageIndex': 0,
        'width': 280.0,
        'height': 160.0,
        'nodes': [
          {
            'type': 'text',
            'text': 'Budget',
            'bounds': {'x': 20.0, 'y': 10.0, 'width': 80.0, 'height': 18.0},
            'style': {
              'fontFamily': 'Calibri',
              'fontSize': 12.0,
              'bold': true,
              'italic': false,
              'underline': false,
              'colorHex': '#000000',
            },
            'range': {'start': 0, 'end': 6},
          },
        ],
        'selectionAnchors': const [],
        'sheetCells': [
          for (var row = 0; row < 4; row++)
            for (var column = 0; column < 4; column++)
              {
                'row': row + 1,
                'column': column + 1,
                'bounds': {
                  'x': column * 70.0,
                  'y': row * 40.0,
                  'width': 70.0,
                  'height': 40.0,
                },
              },
        ],
        'sheetViewport': {
          'window': {
            'startRow': 1,
            'endRow': 4,
            'startColumn': 1,
            'endColumn': 4,
          },
          'effectiveBounds': {
            'startRow': 1,
            'endRow': 40,
            'startColumn': 1,
            'endColumn': 16,
          },
          'visibleRows': [1, 2, 3, 4],
          'visibleColumns': [1, 2, 3, 4],
        },
      });

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

      expect(find.text('A'), findsOneWidget);
      expect(find.text('B'), findsOneWidget);
      expect(find.text('1'), findsOneWidget);
      expect(find.text('4'), findsOneWidget);
    },
  );

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

  testWidgets(
    'sheet viewport consumes pointer scroll signals inside the sheet area',
    (tester) async {
      final page = buildSheetPage(columns: 12, rows: 60);

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

      final rowHeader = find.byKey(const ValueKey('sheet-row-header-8'));
      final initialRowTop = tester.getTopLeft(rowHeader).dy;
      final bodyViewport = find.byKey(const ValueKey('sheet-body-viewport'));
      final bodyCenter = tester.getCenter(bodyViewport);
      final pointer = TestPointer(1, PointerDeviceKind.mouse);

      await tester.sendEventToBinding(pointer.hover(bodyCenter));
      await tester.sendEventToBinding(pointer.scroll(const Offset(0, 160)));
      await tester.pump();

      expect(tester.getTopLeft(rowHeader).dy, lessThan(initialRowTop));
    },
  );

  testWidgets(
    'sheet viewport paints frozen pane overlay when metadata exists',
    (tester) async {
      final page = buildSheetPage(
        columns: 6,
        rows: 12,
        includeFrozenPane: true,
      );

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

      expect(
        find.byKey(const ValueKey('sheet-frozen-overlay')),
        findsOneWidget,
      );
      expect(
        find.byKey(const ValueKey('sheet-frozen-vertical-divider')),
        findsOneWidget,
      );
      expect(
        find.byKey(const ValueKey('sheet-frozen-horizontal-divider')),
        findsOneWidget,
      );
    },
  );

  testWidgets('sheet viewport requests next visible window near scroll edge', (
    tester,
  ) async {
    platform.SheetWindow? requestedWindow;
    final page = buildSheetPage(
      columns: 16,
      rows: 40,
      effectiveEndRow: 120,
      effectiveEndColumn: 24,
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 360,
              height: 280,
              child: SheetViewport(
                page: page,
                onWindowRequest: (window) async {
                  requestedWindow = window;
                },
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    final bodyViewport = find.byKey(const ValueKey('sheet-body-viewport'));
    await tester.drag(
      bodyViewport,
      const Offset(-2200, -2200),
      warnIfMissed: false,
    );
    await tester.pump();

    expect(requestedWindow, isNotNull);
    expect(requestedWindow!.startRow, page.sheetViewport!.window.startRow);
    expect(requestedWindow!.startColumn, page.sheetViewport!.window.startColumn);
    expect(
      requestedWindow!.endRow > page.sheetViewport!.window.endRow ||
          requestedWindow!.endColumn > page.sheetViewport!.window.endColumn,
      isTrue,
    );
  });

  testWidgets(
    'sheet viewport requests previous visible window near leading edge',
    (tester) async {
      platform.SheetWindow? requestedWindow;
      final page = buildSheetPage(
        columns: 16,
        rows: 40,
        startRow: 41,
        startColumn: 9,
        effectiveEndRow: 120,
        effectiveEndColumn: 24,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Center(
              child: SizedBox(
                width: 360,
                height: 280,
                child: SheetViewport(
                  page: page,
                  onWindowRequest: (window) async {
                    requestedWindow = window;
                  },
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      final bodyViewport = find.byKey(const ValueKey('sheet-body-viewport'));
      await tester.drag(
        bodyViewport,
        const Offset(-900, -900),
        warnIfMissed: false,
      );
      await tester.pumpAndSettle();
      await tester.drag(
        bodyViewport,
        const Offset(1800, 1800),
        warnIfMissed: false,
      );
      await tester.pump();

      expect(requestedWindow, isNotNull);
      expect(
        requestedWindow!.startRow < page.sheetViewport!.window.startRow ||
            requestedWindow!.startColumn <
                page.sheetViewport!.window.startColumn,
        isTrue,
      );
      expect(requestedWindow!.endRow, page.sheetViewport!.window.endRow);
      expect(requestedWindow!.endColumn, page.sheetViewport!.window.endColumn);
    },
  );

  testWidgets(
    'sheet viewport expands initial visible window to minimum cell count',
    (tester) async {
      platform.SheetWindow? requestedWindow;
      final page = buildSheetPage(
        columns: 4,
        rows: 8,
        effectiveEndRow: 120,
        effectiveEndColumn: 24,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Center(
              child: SizedBox(
                width: 840,
                height: 640,
                child: SheetViewport(
                  page: page,
                  onWindowRequest: (window) async {
                    requestedWindow = window;
                  },
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pump();

      expect(requestedWindow, isNotNull);
      expect(requestedWindow!.startRow, 1);
      expect(requestedWindow!.startColumn, 1);
      expect(requestedWindow!.endRow, greaterThanOrEqualTo(40));
      expect(requestedWindow!.endColumn, greaterThanOrEqualTo(16));
    },
  );

  testWidgets(
    'sheet viewport expands further when narrow cells do not fill viewport',
    (tester) async {
      platform.SheetWindow? requestedWindow;
      final page = buildSheetPage(
        columns: 16,
        rows: 40,
        cellWidth: 32,
        cellHeight: 20,
        effectiveEndRow: 240,
        effectiveEndColumn: 120,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Center(
              child: SizedBox(
                width: 1100,
                height: 760,
                child: SheetViewport(
                  page: page,
                  onWindowRequest: (window) async {
                    requestedWindow = window;
                  },
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pump();

      expect(requestedWindow, isNotNull);
      expect(requestedWindow!.endColumn, greaterThan(page.sheetViewport!.window.endColumn));
      expect(requestedWindow!.endRow, greaterThan(page.sheetViewport!.window.endRow));
    },
  );

  testWidgets(
    'sheet viewport re-expands when viewport size grows significantly',
    (tester) async {
      final requests = <platform.SheetWindow>[];
      final page = buildSheetPage(
        columns: 4,
        rows: 8,
        effectiveEndRow: 400,
        effectiveEndColumn: 64,
        cellWidth: 32,
        cellHeight: 20,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Center(
              child: SizedBox(
                width: 420,
                height: 320,
                child: SheetViewport(
                  page: page,
                  onWindowRequest: (window) async {
                    requests.add(window);
                  },
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pump();
      await tester.pump();

      expect(requests, isNotEmpty);
      final firstRequest = requests.last;
      requests.clear();
      final expandedPage = buildSheetPage(
        columns: firstRequest.endColumn - firstRequest.startColumn + 1,
        rows: firstRequest.endRow - firstRequest.startRow + 1,
        startRow: firstRequest.startRow,
        startColumn: firstRequest.startColumn,
        effectiveEndRow: 400,
        effectiveEndColumn: 64,
        cellWidth: 32,
        cellHeight: 20,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Center(
              child: SizedBox(
                width: 980,
                height: 720,
                child: SheetViewport(
                  page: expandedPage,
                  onWindowRequest: (window) async {
                    requests.add(window);
                  },
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pump();
      await tester.pump();

      expect(requests, isNotEmpty);
      final secondRequest = requests.last;
      expect(secondRequest.endRow, greaterThanOrEqualTo(firstRequest.endRow));
      expect(
        secondRequest.endColumn,
        greaterThanOrEqualTo(firstRequest.endColumn),
      );
      expect(
        secondRequest.endRow > firstRequest.endRow ||
            secondRequest.endColumn > firstRequest.endColumn,
        isTrue,
      );
    },
  );

  testWidgets(
    'sheet viewport accepts focused cell rect without breaking layout',
    (tester) async {
      final page = buildSheetPage(columns: 12, rows: 80);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Center(
              child: SizedBox(
                width: 320,
                height: 240,
                child: SheetViewport(
                  page: page,
                  focusRect: const Rect.fromLTWH(420, 860, 140, 40),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pump();
      await tester.pump();

      expect(find.byKey(const ValueKey('sheet-body-viewport')), findsOneWidget);
      expect(find.byKey(const ValueKey('sheet-column-header-1')), findsOneWidget);
      expect(find.byKey(const ValueKey('sheet-row-header-1')), findsOneWidget);
    },
  );
}

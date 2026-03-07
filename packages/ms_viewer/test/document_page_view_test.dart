import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

void main() {
  testWidgets('placeholder paint test from mock render model', (tester) async {
    final page = PageRenderModel.fromJson({
      'pageIndex': 0,
      'width': 200.0,
      'height': 300.0,
      'nodes': [
        {
          'type': 'box',
          'bounds': {'x': 12.0, 'y': 18.0, 'width': 80.0, 'height': 24.0},
          'fillColorHex': '#F4E7C5',
          'strokeColorHex': '#D1A954',
          'strokeWidth': 1.0,
        },
        {
          'type': 'text',
          'text': 'Mock paragraph',
          'bounds': {'x': 16.0, 'y': 24.0, 'width': 120.0, 'height': 18.0},
          'style': {
            'fontFamily': 'Calibri',
            'fontSize': 12.0,
            'bold': false,
            'italic': false,
            'colorHex': '#000000',
          },
          'range': {'start': 0, 'end': 14},
        },
      ],
      'selectionAnchors': [
        {'nodeIndex': 1, 'charIndex': 0, 'x': 16.0, 'y': 24.0},
      ],
    });

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 200,
              height: 300,
              child: DocumentPageView(
                page: page,
                highlights: const [Rect.fromLTWH(16, 24, 40, 18)],
              ),
            ),
          ),
        ),
      ),
    );

    expect(find.byType(DocumentPageView), findsOneWidget);
    expect(
      find.descendant(
        of: find.byType(DocumentPageView),
        matching: find.byType(CustomPaint),
      ),
      findsWidgets,
    );
    expect(find.byType(SelectionHighlightOverlay), findsOneWidget);
  });

  testWidgets('renders embedded image nodes as Image widgets', (tester) async {
    final page = PageRenderModel.fromJson({
      'pageIndex': 0,
      'width': 200.0,
      'height': 300.0,
      'nodes': [
        {
          'type': 'image',
          'resourceId': 'word/media/image1.png',
          'description': 'logo',
          'contentType': 'image/png',
          'dataBase64':
              'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+lm2cAAAAASUVORK5CYII=',
          'bounds': {'x': 20.0, 'y': 30.0, 'width': 40.0, 'height': 40.0},
        },
      ],
      'selectionAnchors': const [],
    });

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 200,
              height: 300,
              child: DocumentPageView(page: page),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byType(Image), findsOneWidget);
  });
}

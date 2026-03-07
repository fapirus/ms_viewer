import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

void main() {
  test('render text painter keeps engine lines on a single line', () {
    final node = TextRenderNodeModel.fromJson({
      'type': 'text',
      'text': 'INFINITY TALK',
      'bounds': {'x': 72.0, 'y': 132.0, 'width': 180.0, 'height': 38.0},
      'style': {
        'fontFamily': 'Times New Roman',
        'fontSize': 36.0,
        'bold': true,
        'italic': false,
        'colorHex': '#000000',
      },
      'range': {'start': 0, 'end': 13},
    });

    final painter = buildRenderTextPainter(
      node: node,
      platform: ViewerPlatform.macOs,
      scaleX: 1.0,
      scaleY: 1.0,
    )..layout();

    expect(painter.maxLines, 1);
    expect(painter.didExceedMaxLines, isFalse);
  });

  test('render text painter applies underline decoration from engine style', () {
    final node = TextRenderNodeModel.fromJson({
      'type': 'text',
      'text': '두 가지',
      'bounds': {'x': 72.0, 'y': 132.0, 'width': 120.0, 'height': 38.0},
      'style': {
        'fontFamily': 'Pretendard Medium',
        'fontSize': 30.0,
        'bold': true,
        'italic': false,
        'underline': true,
        'colorHex': '#003296',
      },
      'range': {'start': 0, 'end': 4},
    });

    final painter = buildRenderTextPainter(
      node: node,
      platform: ViewerPlatform.macOs,
      scaleX: 1.0,
      scaleY: 1.0,
    );

    expect(
      painter.text!.style!.decoration,
      TextDecoration.underline,
    );
  });

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
    expect(tester.getSize(find.byType(Image)), const Size(40, 40));
  });

  testWidgets('renders pptx slide nodes with shapes text and image layers', (
    tester,
  ) async {
    final page = PageRenderModel.fromJson({
      'pageIndex': 1,
      'width': 720.0,
      'height': 540.0,
      'nodes': [
        {
          'type': 'box',
          'bounds': {'x': 40.0, 'y': 60.0, 'width': 180.0, 'height': 72.0},
          'fillColorHex': '#FFAA00',
          'strokeColorHex': '#333333',
          'strokeWidth': 1.0,
        },
        {
          'type': 'text',
          'text': 'Quarterly Results',
          'bounds': {'x': 66.0, 'y': 24.0, 'width': 220.0, 'height': 28.0},
          'style': {
            'fontFamily': 'Aptos',
            'fontSize': 24.0,
            'bold': true,
            'italic': false,
            'colorHex': '#000000',
          },
          'range': {'start': 0, 'end': 17},
        },
        {
          'type': 'image',
          'resourceId': 'ppt/media/image1.png',
          'description': 'Fixture image',
          'contentType': 'image/png',
          'dataBase64':
              'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+lm2cAAAAASUVORK5CYII=',
          'bounds': {'x': 300.0, 'y': 120.0, 'width': 100.0, 'height': 75.0},
        },
      ],
      'selectionAnchors': const [
        {'nodeIndex': 1, 'charIndex': 0, 'x': 66.0, 'y': 24.0},
        {'nodeIndex': 1, 'charIndex': 17, 'x': 286.0, 'y': 24.0},
      ],
    });

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 360,
              height: 270,
              child: DocumentPageView(
                page: page,
                highlights: const [Rect.fromLTWH(66, 24, 80, 28)],
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byType(DocumentPageView), findsOneWidget);
    expect(find.byType(CustomPaint), findsWidgets);
    expect(find.byType(Image), findsOneWidget);
    expect(find.byType(SelectionHighlightOverlay), findsOneWidget);
    expect(tester.getSize(find.byType(Image)), const Size(50, 37.5));
  });

  testWidgets('renders gradient background boxes without crashing', (tester) async {
    final page = PageRenderModel.fromJson({
      'pageIndex': 0,
      'width': 720.0,
      'height': 540.0,
      'nodes': [
        {
          'type': 'box',
          'bounds': {'x': 0.0, 'y': 0.0, 'width': 720.0, 'height': 540.0},
          'fillColorHex': '#003EA7',
          'gradientEndColorHex': '#70AD47',
          'gradientAngleDegrees': 62.0,
          'strokeColorHex': null,
          'strokeWidth': 0.0,
        },
      ],
      'selectionAnchors': const [],
    });

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: SizedBox(width: 360, height: 270, child: DocumentPageView(page: page)),
          ),
        ),
      ),
    );

    expect(find.byType(DocumentPageView), findsOneWidget);
    expect(find.byType(CustomPaint), findsWidgets);
  });

  test('render text painter supports gradient text styles', () {
    final node = TextRenderNodeModel.fromJson({
      'type': 'text',
      'text': 'Theme Title',
      'bounds': {'x': 32.0, 'y': 48.0, 'width': 240.0, 'height': 54.0},
      'style': {
        'fontFamily': '맑은 고딕',
        'fontSize': 48.0,
        'bold': true,
        'italic': false,
        'colorHex': '#003EA7',
        'gradientEndColorHex': '#70AD47',
        'gradientAngleDegrees': 32.0,
      },
      'range': {'start': 0, 'end': 11},
    });

    final painter = buildRenderTextPainter(
      node: node,
      platform: ViewerPlatform.macOs,
      scaleX: 1.0,
      scaleY: 1.0,
    )..layout();

    expect(painter.maxLines, 1);
    expect(painter.didExceedMaxLines, isFalse);
  });
}

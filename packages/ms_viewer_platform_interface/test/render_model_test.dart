import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

void main() {
  test('page render model decodes text node and selection anchor', () {
    final page = PageRenderModel.fromJson({
      'pageIndex': 0,
      'width': 595.0,
      'height': 842.0,
      'nodes': [
        {
          'type': 'text',
          'text': 'Hello',
          'bounds': {'x': 24.0, 'y': 32.0, 'width': 120.0, 'height': 18.0},
          'style': {
            'fontFamily': 'Calibri',
            'fontSize': 11.0,
            'bold': false,
            'italic': false,
            'colorHex': '#000000',
          },
          'range': {'start': 0, 'end': 5},
        },
      ],
      'selectionAnchors': [
        {'nodeIndex': 0, 'charIndex': 0, 'x': 24.0, 'y': 32.0},
      ],
    });

    expect(page.pageIndex, 0);
    expect(page.nodes.single, isA<TextRenderNodeModel>());
    expect(page.selectionAnchors.single.charIndex, 0);
  });

  test('page render model decodes box node', () {
    final page = PageRenderModel.fromJson({
      'pageIndex': 1,
      'width': 1024.0,
      'height': 768.0,
      'nodes': [
        {
          'type': 'box',
          'bounds': {'x': 0.0, 'y': 0.0, 'width': 1024.0, 'height': 768.0},
          'fillColorHex': '#FFFFFF',
          'strokeColorHex': null,
          'strokeWidth': 0.0,
        },
      ],
      'selectionAnchors': [],
    });

    expect(page.nodes.single, isA<BoxRenderNodeModel>());
  });

  test('page render model decodes gradient box node', () {
    final page = PageRenderModel.fromJson({
      'pageIndex': 1,
      'width': 1024.0,
      'height': 768.0,
      'nodes': [
        {
          'type': 'box',
          'bounds': {'x': 0.0, 'y': 0.0, 'width': 1024.0, 'height': 768.0},
          'fillColorHex': '#003EA7',
          'gradientEndColorHex': '#70AD47',
          'gradientAngleDegrees': 62.0,
          'strokeColorHex': null,
          'strokeWidth': 0.0,
        },
      ],
      'selectionAnchors': [],
    });

    final node = page.nodes.single as BoxRenderNodeModel;
    expect(node.fillColorHex, '#003EA7');
    expect(node.gradientEndColorHex, '#70AD47');
    expect(node.gradientAngleDegrees, 62.0);
  });

  test('page render model decodes image node payload', () {
    final page = PageRenderModel.fromJson({
      'pageIndex': 2,
      'width': 640.0,
      'height': 480.0,
      'nodes': [
        {
          'type': 'image',
          'resourceId': 'word/media/image1.png',
          'description': 'logo',
          'contentType': 'image/png',
          'dataBase64': 'aGVsbG8=',
          'bounds': {'x': 10.0, 'y': 20.0, 'width': 100.0, 'height': 80.0},
          'crop': {'left': 0.1, 'top': 0.05, 'right': 0.2, 'bottom': 0.15},
          'flipHorizontal': true,
          'flipVertical': false,
        },
      ],
      'selectionAnchors': [],
    });

    final node = page.nodes.single as ImageRenderNodeModel;
    expect(node.resourceId, 'word/media/image1.png');
    expect(node.contentType, 'image/png');
    expect(node.dataBase64, 'aGVsbG8=');
    expect(node.crop!.left, 0.1);
    expect(node.flipHorizontal, isTrue);
    expect(node.flipVertical, isFalse);
  });

  test('page render model decodes box corner radius field', () {
    final page = PageRenderModel.fromJson({
      'pageIndex': 1,
      'width': 1024.0,
      'height': 768.0,
      'nodes': [
        {
          'type': 'box',
          'bounds': {'x': 0.0, 'y': 0.0, 'width': 300.0, 'height': 120.0},
          'fillColorHex': '#FFFFFF80',
          'strokeColorHex': null,
          'strokeWidth': 0.0,
          'cornerRadius': 18.0,
        },
      ],
      'selectionAnchors': [],
    });

    final node = page.nodes.single as BoxRenderNodeModel;
    expect(node.fillColorHex, '#FFFFFF80');
    expect(node.cornerRadius, 18.0);
  });

  test('page render model decodes gradient text style fields', () {
    final page = PageRenderModel.fromJson({
      'pageIndex': 0,
      'width': 720.0,
      'height': 540.0,
      'nodes': [
        {
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
        },
      ],
      'selectionAnchors': const [],
    });

    final node = page.nodes.single as TextRenderNodeModel;
    expect(node.style.colorHex, '#003EA7');
    expect(node.style.gradientEndColorHex, '#70AD47');
    expect(node.style.gradientAngleDegrees, 32.0);
  });

  test('page render model decodes underline text style field', () {
    final page = PageRenderModel.fromJson({
      'pageIndex': 0,
      'width': 720.0,
      'height': 540.0,
      'nodes': [
        {
          'type': 'text',
          'text': '두 가지',
          'bounds': {'x': 32.0, 'y': 48.0, 'width': 120.0, 'height': 36.0},
          'style': {
            'fontFamily': 'Pretendard Medium',
            'fontSize': 30.0,
            'bold': true,
            'italic': false,
            'underline': true,
            'colorHex': '#003296',
          },
          'range': {'start': 0, 'end': 4},
        },
      ],
      'selectionAnchors': const [],
    });

    final node = page.nodes.single as TextRenderNodeModel;
    expect(node.style.underline, isTrue);
  });

  test('page render model decodes xlsx sheet viewport metadata', () {
    final page = PageRenderModel.fromJson({
      'pageIndex': 0,
      'width': 640.0,
      'height': 360.0,
      'nodes': const [],
      'selectionAnchors': const [],
      'sheetViewport': {
        'window': {
          'startRow': 3,
          'endRow': 22,
          'startColumn': 2,
          'endColumn': 9,
        },
        'effectiveBounds': {
          'startRow': 1,
          'endRow': 120,
          'startColumn': 1,
          'endColumn': 24,
        },
        'frozenPane': {
          'frozenRows': 1,
          'frozenColumns': 2,
          'topLeftCell': 'C2',
        },
        'visibleRows': [3, 4, 5, 6],
        'visibleColumns': [2, 3, 4, 5, 6],
      },
      'sheetCells': [
        {
          'row': 3,
          'column': 2,
          'bounds': {'x': 0.0, 'y': 0.0, 'width': 120.0, 'height': 24.0},
        },
      ],
    });

    expect(page.sheetViewport, isNotNull);
    expect(page.sheetViewport!.window.startRow, 3);
    expect(page.sheetViewport!.effectiveBounds.endColumn, 24);
    expect(page.sheetViewport!.frozenPane!.frozenRows, 1);
    expect(page.sheetViewport!.frozenPane!.topLeftCell, 'C2');
    expect(page.sheetViewport!.visibleRows, [3, 4, 5, 6]);
    expect(page.sheetViewport!.visibleColumns, [2, 3, 4, 5, 6]);
    expect(page.sheetCells.single.row, 3);
    expect(page.sheetCells.single.column, 2);
  });
}

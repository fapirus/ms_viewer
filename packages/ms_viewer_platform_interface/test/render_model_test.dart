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
        },
      ],
      'selectionAnchors': [],
    });

    final node = page.nodes.single as ImageRenderNodeModel;
    expect(node.resourceId, 'word/media/image1.png');
    expect(node.contentType, 'image/png');
    expect(node.dataBase64, 'aGVsbG8=');
  });
}

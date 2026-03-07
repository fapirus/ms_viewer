import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

List<PageRenderModel> buildFixturePreviewPages(String fixtureId) {
  switch (fixtureId) {
    case 'docx_plain_text':
      return [
        _page([
          _text('Plain text fixture document.', 52, 72, 220, 18),
          _text('This preview is bundled in the demo app.', 52, 98, 260, 16),
        ]),
      ];
    case 'docx_styles_lists':
      return [
        _page([
          _text(
            'Styled intro',
            52,
            72,
            180,
            22,
            bold: true,
            colorHex: '#1D4ED8',
          ),
          _bullet('Bullet item', 64, 116),
          _bullet('Nested number', 88, 146, bullet: '1.'),
        ]),
      ];
    case 'docx_tables_images':
      return [
        _page([
          _box(52, 72, 220, 84, fill: '#F8FAFC', stroke: '#CBD5E1'),
          _text('A1', 68, 96, 40, 16),
          _text('B1', 176, 96, 40, 16),
          _image('Fixture image', 52, 186, 220, 128),
        ]),
      ];
    case 'docx_multi_section':
      return [
        _page([
          _text('Section one', 52, 72, 180, 18),
          _text('Fixture header', 52, 28, 160, 14, colorHex: '#64748B'),
        ]),
        _page([
          _text('Section two', 52, 72, 180, 18),
          _text('Fixture footer', 52, 700, 160, 14, colorHex: '#64748B'),
        ]),
      ];
    default:
      return [buildImportedPreviewPage(title: fixtureId, extension: 'docx')];
  }
}

PageRenderModel buildImportedPreviewPage({
  required String title,
  required String extension,
}) {
  final label = extension.toUpperCase();
  final note = switch (extension) {
    'docx' => 'DOCX preview shell. Engine-backed page fetch is next.',
    'pptx' => 'PPTX phase has not started yet.',
    'xlsx' => 'XLSX phase has not started yet.',
    _ => 'Unsupported preview type.',
  };

  return _page([
    _box(44, 52, 300, 96, fill: '#DBEAFE', stroke: '#60A5FA'),
    _text('$label document', 60, 78, 220, 22, bold: true, colorHex: '#1D4ED8'),
    _text(title, 60, 106, 260, 18),
    _text(note, 44, 188, 320, 18, colorHex: '#475569'),
  ]);
}

PageRenderModel _page(List<Map<String, Object?>> nodes) {
  return PageRenderModel.fromJson({
    'pageIndex': 0,
    'width': 595.0,
    'height': 842.0,
    'nodes': nodes,
    'selectionAnchors': const [],
  });
}

Map<String, Object?> _text(
  String text,
  double x,
  double y,
  double width,
  double height, {
  bool bold = false,
  String colorHex = '#0F172A',
}) {
  return {
    'type': 'text',
    'text': text,
    'bounds': {'x': x, 'y': y, 'width': width, 'height': height},
    'style': {
      'fontFamily': 'Calibri',
      'fontSize': bold ? 14.0 : 12.0,
      'bold': bold,
      'italic': false,
      'colorHex': colorHex,
    },
    'range': {'start': 0, 'end': text.length},
  };
}

Map<String, Object?> _box(
  double x,
  double y,
  double width,
  double height, {
  required String fill,
  required String stroke,
}) {
  return {
    'type': 'box',
    'bounds': {'x': x, 'y': y, 'width': width, 'height': height},
    'fillColorHex': fill,
    'strokeColorHex': stroke,
    'strokeWidth': 1.0,
  };
}

Map<String, Object?> _image(
  String label,
  double x,
  double y,
  double width,
  double height,
) {
  return {
    'type': 'image',
    'resourceId': label.toLowerCase().replaceAll(' ', '_'),
    'description': label,
    'bounds': {'x': x, 'y': y, 'width': width, 'height': height},
  };
}

Map<String, Object?> _bullet(
  String text,
  double x,
  double y, {
  String bullet = '•',
}) {
  return _text('$bullet  $text', x, y, 220, 16);
}

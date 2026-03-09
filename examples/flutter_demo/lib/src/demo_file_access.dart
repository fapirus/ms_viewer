import 'package:file_picker/file_picker.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:path/path.dart' as p;

import 'demo_document.dart';
import 'demo_page_models.dart';

typedef DemoFilePicker = Future<List<String>> Function();

Future<List<String>> pickDemoFiles() async {
  final result = await FilePicker.platform.pickFiles(
    allowMultiple: true,
    type: FileType.custom,
    allowedExtensions: const ['docx', 'pptx', 'xlsx'],
  );
  if (result == null) {
    return const [];
  }

  return result.files
      .map((file) => file.path)
      .whereType<String>()
      .toList(growable: false);
}

DemoDocumentEntry buildImportedEntry(String path, DemoDocumentOrigin origin) {
  final extension = p.extension(path).replaceFirst('.', '').toLowerCase();
  final title = p.basename(path);
  final kind = switch (extension) {
    'pptx' => DocumentKind.pptx,
    'xlsx' => DocumentKind.xlsx,
    _ => DocumentKind.docx,
  };
  final note = switch ((origin, extension)) {
    (DemoDocumentOrigin.picked, 'docx') =>
      'External DOCX file. Real engine open is wired in the desktop demo.',
    (DemoDocumentOrigin.picked, 'pptx') =>
      'External PPTX file. Real engine open is wired in the desktop demo.',
    (DemoDocumentOrigin.picked, 'xlsx') =>
      'External XLSX file. Real engine open is wired in the desktop demo.',
    (DemoDocumentOrigin.dropped, 'docx') =>
      'External DOCX file. Real engine open is wired in the desktop demo.',
    (DemoDocumentOrigin.dropped, 'pptx') =>
      'External PPTX file. Real engine open is wired in the desktop demo.',
    (DemoDocumentOrigin.dropped, 'xlsx') =>
      'External XLSX file. Real engine open is wired in the desktop demo.',
    (_, 'xlsx') =>
      'External XLSX file. Real engine open will be wired in the desktop demo.',
    (_, 'pptx') =>
      'External PPTX file. Real engine open will be wired in the desktop demo.',
    (_, 'docx') =>
      'External DOCX file. Real engine open is wired in the desktop demo.',
    _ => 'Unknown document type.',
  };

  return DemoDocumentEntry(
    id: '${origin.name}:$path',
    title: title,
    kind: kind,
    origin: origin,
    previewPages: [
      buildImportedPreviewPage(title: title, extension: extension),
    ],
    tags: [extension.isEmpty ? 'file' : extension, origin.name],
    sourceLabel: origin == DemoDocumentOrigin.picked
        ? 'Picked file'
        : 'Dropped file',
    statusLabel: 'Ready',
    location: path,
    note: note,
  );
}

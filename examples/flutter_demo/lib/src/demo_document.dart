import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;

enum DemoDocumentOrigin { fixture, picked, dropped }

class DemoDocumentEntry {
  const DemoDocumentEntry({
    required this.id,
    required this.title,
    required this.kind,
    required this.origin,
    required this.previewPages,
    required this.tags,
    required this.sourceLabel,
    required this.statusLabel,
    this.location,
    this.requiresPassword = false,
    this.note,
  });

  final String id;
  final String title;
  final DocumentKind kind;
  final DemoDocumentOrigin origin;
  final List<platform.PageRenderModel> previewPages;
  final List<String> tags;
  final String sourceLabel;
  final String statusLabel;
  final String? location;
  final bool requiresPassword;
  final String? note;

  DocumentDescriptor get descriptor => DocumentDescriptor(
    id: id,
    kind: kind,
    title: title,
    pageCount: previewPages.isEmpty ? 1 : previewPages.length,
  );
}

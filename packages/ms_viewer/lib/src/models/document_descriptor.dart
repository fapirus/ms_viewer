import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;

enum DocumentKind { docx, pptx, xlsx }

class DocumentDescriptor {
  const DocumentDescriptor({
    required this.id,
    required this.kind,
    required this.title,
    required this.pageCount,
  });

  final String id;
  final DocumentKind kind;
  final String title;
  final int pageCount;

  factory DocumentDescriptor.fromOpenDocumentSuccess(
    platform.OpenDocumentSuccess success,
  ) {
    return DocumentDescriptor(
      id: success.documentId,
      kind: switch (success.kind) {
        platform.DocumentKind.docx => DocumentKind.docx,
        platform.DocumentKind.pptx => DocumentKind.pptx,
        platform.DocumentKind.xlsx => DocumentKind.xlsx,
      },
      title: success.title,
      pageCount: success.pageCount,
    );
  }
}

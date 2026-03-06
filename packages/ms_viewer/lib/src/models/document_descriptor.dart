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
}

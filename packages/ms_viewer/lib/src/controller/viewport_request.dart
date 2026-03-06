import '../models/document_descriptor.dart';

class ViewportRequest {
  const ViewportRequest.docx({required this.index})
    : kind = DocumentKind.docx,
      sheetWindow = null;

  const ViewportRequest.pptx({required this.index})
    : kind = DocumentKind.pptx,
      sheetWindow = null;

  const ViewportRequest.xlsx({required this.index, required this.sheetWindow})
    : kind = DocumentKind.xlsx;

  final DocumentKind kind;
  final int index;
  final SheetWindow? sheetWindow;
}

class SheetWindow {
  const SheetWindow({
    required this.startRow,
    required this.endRow,
    required this.startColumn,
    required this.endColumn,
  });

  final int startRow;
  final int endRow;
  final int startColumn;
  final int endColumn;
}

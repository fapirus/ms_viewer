class SearchResult {
  const SearchResult({
    required this.query,
    required this.pageIndex,
    required this.start,
    required this.end,
    required this.preview,
    required this.sheetCell,
  });

  final String query;
  final int pageIndex;
  final int start;
  final int end;
  final String preview;
  final SearchSheetCell? sheetCell;
}

class SearchSheetCell {
  const SearchSheetCell({required this.row, required this.column});

  final int row;
  final int column;
}

class SearchSheetRange {
  const SearchSheetRange({
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

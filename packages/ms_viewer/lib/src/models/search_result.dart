class SearchResult {
  const SearchResult({
    required this.query,
    required this.pageIndex,
    required this.start,
    required this.end,
    required this.preview,
  });

  final String query;
  final int pageIndex;
  final int start;
  final int end;
  final String preview;
}

class SearchResult {
  const SearchResult({
    required this.query,
    required this.pageIndex,
    required this.preview,
  });

  final String query;
  final int pageIndex;
  final String preview;
}

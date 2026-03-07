import 'package:flutter/material.dart';

import '../models/search_result.dart';

class SearchResultList extends StatelessWidget {
  const SearchResultList({
    super.key,
    required this.results,
    required this.currentIndex,
    this.onTap,
  });

  final List<SearchResult> results;
  final int currentIndex;
  final ValueChanged<int>? onTap;

  @override
  Widget build(BuildContext context) {
    if (results.isEmpty) {
      return const Center(child: Text('No results'));
    }

    return ListView.builder(
      itemCount: results.length,
      itemBuilder: (context, index) {
        final result = results[index];
        final selected = index == currentIndex;
        return ListTile(
          selected: selected,
          title: Text('Page ${result.pageIndex + 1}'),
          subtitle: Text(result.preview),
          onTap: onTap == null ? null : () => onTap!(index),
        );
      },
    );
  }
}

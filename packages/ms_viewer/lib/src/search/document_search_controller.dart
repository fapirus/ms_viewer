import 'package:flutter/foundation.dart';

import '../models/search_result.dart';

class DocumentSearchController extends ChangeNotifier {
  String _query = '';
  List<SearchResult> _results = const [];
  int _currentIndex = -1;

  String get query => _query;
  List<SearchResult> get results => List.unmodifiable(_results);
  int get currentIndex => _currentIndex;
  SearchResult? get currentResult =>
      _currentIndex >= 0 && _currentIndex < _results.length
          ? _results[_currentIndex]
          : null;

  void updateResults({
    required String query,
    required List<SearchResult> results,
  }) {
    _query = query;
    _results = List.unmodifiable(results);
    _currentIndex = results.isEmpty ? -1 : 0;
    notifyListeners();
  }

  void clear() {
    _query = '';
    _results = const [];
    _currentIndex = -1;
    notifyListeners();
  }

  void select(int index) {
    if (index < 0 || index >= _results.length) {
      return;
    }
    _currentIndex = index;
    notifyListeners();
  }

  void next() {
    if (_results.isEmpty) {
      return;
    }
    _currentIndex = (_currentIndex + 1) % _results.length;
    notifyListeners();
  }

  void previous() {
    if (_results.isEmpty) {
      return;
    }
    _currentIndex = (_currentIndex - 1 + _results.length) % _results.length;
    notifyListeners();
  }
}

import 'dart:ui';

import 'package:flutter/foundation.dart';

class SelectionDragController extends ChangeNotifier {
  Offset? _dragStart;
  Offset? _dragCurrent;

  Offset? get dragStart => _dragStart;
  Offset? get dragCurrent => _dragCurrent;
  bool get isDragging => _dragStart != null && _dragCurrent != null;

  Rect? get selectionRect {
    final start = _dragStart;
    final current = _dragCurrent;
    if (start == null || current == null) {
      return null;
    }
    return Rect.fromPoints(start, current);
  }

  void start(Offset position) {
    _dragStart = position;
    _dragCurrent = position;
    notifyListeners();
  }

  void update(Offset position) {
    if (_dragStart == null) {
      return;
    }
    _dragCurrent = position;
    notifyListeners();
  }

  void clear() {
    _dragStart = null;
    _dragCurrent = null;
    notifyListeners();
  }
}

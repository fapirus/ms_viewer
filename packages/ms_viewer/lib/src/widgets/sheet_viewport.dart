import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import 'render_node_canvas.dart';

class SheetViewport extends StatefulWidget {
  const SheetViewport({
    super.key,
    required this.page,
    this.highlights = const [],
    this.onSelectionStart,
    this.onSelectionUpdate,
    this.onSelectionEnd,
    this.onWindowRequest,
  });

  final PageRenderModel page;
  final List<Rect> highlights;
  final ValueChanged<Offset>? onSelectionStart;
  final ValueChanged<Offset>? onSelectionUpdate;
  final VoidCallback? onSelectionEnd;
  final Future<void> Function(SheetWindow window)? onWindowRequest;

  @override
  State<SheetViewport> createState() => _SheetViewportState();
}

class _SheetViewportState extends State<SheetViewport> {
  static const double _cornerExtent = 56;
  static const double _headerExtent = 36;
  static const Color _headerBackground = Color(0xFFF3F6FA);
  static const Color _headerBorder = Color(0xFFD0D7DE);
  static const double _windowShiftRatio = 0.5;
  static const double _requestThreshold = 0.72;
  static const int _minimumVisibleRows = 24;
  static const int _minimumVisibleColumns = 10;
  static const double _targetRowPixels = 30;
  static const double _targetColumnPixels = 108;

  final ScrollController _horizontalBodyController = ScrollController();
  final ScrollController _verticalBodyController = ScrollController();
  final ScrollController _horizontalHeaderController = ScrollController();
  final ScrollController _verticalHeaderController = ScrollController();

  bool _syncingHorizontal = false;
  bool _syncingVertical = false;
  bool _windowRequestInFlight = false;
  int _pendingColumnShift = 0;
  int _pendingRowShift = 0;
  _SheetViewportMetrics? _latestMetrics;
  bool _initialWindowExpansionScheduled = false;

  @override
  void initState() {
    super.initState();
    _horizontalBodyController.addListener(_syncHorizontalOffset);
    _verticalBodyController.addListener(_syncVerticalOffset);
  }

  @override
  void didUpdateWidget(covariant SheetViewport oldWidget) {
    super.didUpdateWidget(oldWidget);
    final oldWindow = oldWidget.page.sheetViewport?.window;
    final newWindow = widget.page.sheetViewport?.window;
    if (oldWindow != null &&
        newWindow != null &&
        !_sameWindowBounds(oldWindow, newWindow)) {
      _initialWindowExpansionScheduled = false;
      final oldMetrics = _SheetViewportMetrics.fromPage(oldWidget.page);
      final horizontalAdjustment = oldMetrics.extentForLeadingColumns(
        _pendingColumnShift,
      );
      final verticalAdjustment = oldMetrics.extentForLeadingRows(
        _pendingRowShift,
      );
      _pendingColumnShift = 0;
      _pendingRowShift = 0;
      _windowRequestInFlight = false;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) {
          return;
        }
        if (horizontalAdjustment > 0 && _horizontalBodyController.hasClients) {
          _horizontalBodyController.jumpTo(
            (_horizontalBodyController.offset - horizontalAdjustment).clamp(
              0.0,
              _horizontalBodyController.position.maxScrollExtent,
            ),
          );
        }
        if (verticalAdjustment > 0 && _verticalBodyController.hasClients) {
          _verticalBodyController.jumpTo(
            (_verticalBodyController.offset - verticalAdjustment).clamp(
              0.0,
              _verticalBodyController.position.maxScrollExtent,
            ),
          );
        }
      });
    }
  }

  @override
  void dispose() {
    _horizontalBodyController.removeListener(_syncHorizontalOffset);
    _verticalBodyController.removeListener(_syncVerticalOffset);
    _horizontalBodyController.dispose();
    _verticalBodyController.dispose();
    _horizontalHeaderController.dispose();
    _verticalHeaderController.dispose();
    super.dispose();
  }

  void _syncHorizontalOffset() {
    if (_syncingHorizontal || !_horizontalHeaderController.hasClients) {
      return;
    }
    _syncingHorizontal = true;
    final offset = _horizontalBodyController.offset.clamp(
      0.0,
      _horizontalHeaderController.position.maxScrollExtent,
    );
    _horizontalHeaderController.jumpTo(offset);
    _syncingHorizontal = false;
    _maybeRequestWindow();
    if (mounted) {
      setState(() {});
    }
  }

  void _syncVerticalOffset() {
    if (_syncingVertical || !_verticalHeaderController.hasClients) {
      return;
    }
    _syncingVertical = true;
    final offset = _verticalBodyController.offset.clamp(
      0.0,
      _verticalHeaderController.position.maxScrollExtent,
    );
    _verticalHeaderController.jumpTo(offset);
    _syncingVertical = false;
    _maybeRequestWindow();
    if (mounted) {
      setState(() {});
    }
  }

  void _maybeRequestWindow() {
    final callback = widget.onWindowRequest;
    final sheetViewport = widget.page.sheetViewport;
    final metrics = _latestMetrics;
    if (callback == null ||
        sheetViewport == null ||
        metrics == null ||
        _windowRequestInFlight) {
      return;
    }

    final currentWindow = sheetViewport.window;
    final effectiveBounds = sheetViewport.effectiveBounds;
    final rowCount = currentWindow.endRow - currentWindow.startRow + 1;
    final columnCount = currentWindow.endColumn - currentWindow.startColumn + 1;
    var nextStartRow = currentWindow.startRow;
    var nextStartColumn = currentWindow.startColumn;

    if (_horizontalBodyController.hasClients &&
        _horizontalBodyController.position.maxScrollExtent > 0 &&
        _horizontalBodyController.offset >=
            _horizontalBodyController.position.maxScrollExtent *
                _requestThreshold &&
        currentWindow.endColumn < effectiveBounds.endColumn) {
      final shift = math.max(1, (columnCount * _windowShiftRatio).round());
      final maxStart = math.max(
        effectiveBounds.startColumn,
        effectiveBounds.endColumn - columnCount + 1,
      );
      nextStartColumn = math.min(currentWindow.startColumn + shift, maxStart);
    }

    if (_verticalBodyController.hasClients &&
        _verticalBodyController.position.maxScrollExtent > 0 &&
        _verticalBodyController.offset >=
            _verticalBodyController.position.maxScrollExtent *
                _requestThreshold &&
        currentWindow.endRow < effectiveBounds.endRow) {
      final shift = math.max(1, (rowCount * _windowShiftRatio).round());
      final maxStart = math.max(
        effectiveBounds.startRow,
        effectiveBounds.endRow - rowCount + 1,
      );
      nextStartRow = math.min(currentWindow.startRow + shift, maxStart);
    }

    if (nextStartRow == currentWindow.startRow &&
        nextStartColumn == currentWindow.startColumn) {
      return;
    }

    _windowRequestInFlight = true;
    _pendingColumnShift = nextStartColumn - currentWindow.startColumn;
    _pendingRowShift = nextStartRow - currentWindow.startRow;
    unawaited(
      callback(
        SheetWindow(
          startRow: nextStartRow,
          endRow: nextStartRow + rowCount - 1,
          startColumn: nextStartColumn,
          endColumn: nextStartColumn + columnCount - 1,
        ),
      ).catchError((_) {
        _windowRequestInFlight = false;
        _pendingColumnShift = 0;
        _pendingRowShift = 0;
      }),
    );
  }

  @override
  Widget build(BuildContext context) {
    final metrics = _SheetViewportMetrics.fromPage(widget.page);
    _latestMetrics = metrics;

    return LayoutBuilder(
      builder: (context, constraints) {
        _scheduleMinimumWindowExpansion(constraints.biggest);
        return DecoratedBox(
          decoration: BoxDecoration(
            color: const Color(0xFFF7F9FC),
            border: Border.all(color: _headerBorder),
            borderRadius: BorderRadius.circular(14),
          ),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(14),
            child: Column(
              children: [
                SizedBox(
                  height: _headerExtent,
                  child: Row(
                    children: [
                      _buildCornerCell(),
                      Expanded(child: _buildColumnHeaders(metrics)),
                    ],
                  ),
                ),
                const Divider(height: 1, thickness: 1, color: _headerBorder),
                Expanded(
                  child: Row(
                    children: [
                      SizedBox(
                        width: _cornerExtent,
                        child: _buildRowHeaders(metrics),
                      ),
                      const VerticalDivider(
                        width: 1,
                        thickness: 1,
                        color: _headerBorder,
                      ),
                      Expanded(child: _buildScrollableBody(metrics)),
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  void _scheduleMinimumWindowExpansion(Size viewportSize) {
    if (_initialWindowExpansionScheduled) {
      return;
    }
    final callback = widget.onWindowRequest;
    final viewport = widget.page.sheetViewport;
    if (callback == null || viewport == null) {
      return;
    }

    final currentWindow = viewport.window;
    final effectiveBounds = viewport.effectiveBounds;
    final desiredRowCount = math.max(
      _minimumVisibleRows,
      ((viewportSize.height - _headerExtent) / _targetRowPixels).ceil(),
    );
    final desiredColumnCount = math.max(
      _minimumVisibleColumns,
      ((viewportSize.width - _cornerExtent) / _targetColumnPixels).ceil(),
    );
    final currentRowCount = currentWindow.endRow - currentWindow.startRow + 1;
    final currentColumnCount =
        currentWindow.endColumn - currentWindow.startColumn + 1;
    final targetEndRow = math.min(
      effectiveBounds.endRow,
      currentWindow.startRow + desiredRowCount - 1,
    );
    final targetEndColumn = math.min(
      effectiveBounds.endColumn,
      currentWindow.startColumn + desiredColumnCount - 1,
    );

    if (targetEndRow <= currentWindow.endRow &&
        targetEndColumn <= currentWindow.endColumn) {
      _initialWindowExpansionScheduled = true;
      return;
    }

    if (currentRowCount >= desiredRowCount &&
        currentColumnCount >= desiredColumnCount) {
      _initialWindowExpansionScheduled = true;
      return;
    }

    _initialWindowExpansionScheduled = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || _windowRequestInFlight) {
        return;
      }
      _windowRequestInFlight = true;
      unawaited(
        callback(
          SheetWindow(
            startRow: currentWindow.startRow,
            endRow: targetEndRow,
            startColumn: currentWindow.startColumn,
            endColumn: targetEndColumn,
          ),
        ).catchError((_) {
          _windowRequestInFlight = false;
        }),
      );
    });
  }

  Widget _buildCornerCell() {
    return Container(
      key: const ValueKey('sheet-corner-cell'),
      width: _cornerExtent,
      height: _headerExtent,
      color: _headerBackground,
      alignment: Alignment.center,
      child: const Icon(Icons.table_rows_outlined, size: 18),
    );
  }

  Widget _buildColumnHeaders(_SheetViewportMetrics metrics) {
    return ColoredBox(
      color: _headerBackground,
      child: SingleChildScrollView(
        controller: _horizontalHeaderController,
        physics: const NeverScrollableScrollPhysics(),
        scrollDirection: Axis.horizontal,
        child: SizedBox(
          width: widget.page.width,
          height: _headerExtent,
          child: Stack(
            children: [
              for (var index = 0; index < metrics.columns.length; index++)
                Positioned(
                  key: ValueKey('sheet-column-header-${index + 1}'),
                  left: metrics.columns[index].start,
                  top: 0,
                  width: metrics.columns[index].extent,
                  height: _headerExtent,
                  child: _SheetHeaderCell(
                    label: _columnLabel(metrics.visibleColumns[index]),
                    alignment: Alignment.center,
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildRowHeaders(_SheetViewportMetrics metrics) {
    return ColoredBox(
      color: _headerBackground,
      child: SingleChildScrollView(
        controller: _verticalHeaderController,
        physics: const NeverScrollableScrollPhysics(),
        scrollDirection: Axis.vertical,
        child: SizedBox(
          width: _cornerExtent,
          height: widget.page.height,
          child: Stack(
            children: [
              for (var index = 0; index < metrics.rows.length; index++)
                Positioned(
                  key: ValueKey('sheet-row-header-${index + 1}'),
                  left: 0,
                  top: metrics.rows[index].start,
                  width: _cornerExtent,
                  height: metrics.rows[index].extent,
                  child: _SheetHeaderCell(
                    label: '${metrics.visibleRows[index]}',
                    alignment: Alignment.centerRight,
                    padding: const EdgeInsets.only(right: 12),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildScrollableBody(_SheetViewportMetrics metrics) {
    final frozenPane = widget.page.sheetViewport?.frozenPane;
    final frozenColumnCount = math.min(
      metrics.columns.length,
      frozenPane?.frozenColumns ?? 0,
    );
    final frozenRowCount = math.min(
      metrics.rows.length,
      frozenPane?.frozenRows ?? 0,
    );
    final frozenWidth = metrics.extentForLeadingColumns(frozenColumnCount);
    final frozenHeight = metrics.extentForLeadingRows(frozenRowCount);

    return Stack(
      children: [
        KeyedSubtree(
          key: const ValueKey('sheet-body-viewport'),
          child: Scrollbar(
            controller: _verticalBodyController,
            thumbVisibility: true,
            child: Scrollbar(
              controller: _horizontalBodyController,
              thumbVisibility: true,
              notificationPredicate: (notification) => notification.depth == 1,
              child: SingleChildScrollView(
                controller: _verticalBodyController,
                scrollDirection: Axis.vertical,
                child: SingleChildScrollView(
                  controller: _horizontalBodyController,
                  scrollDirection: Axis.horizontal,
                  child: RenderNodeCanvas(
                    key: const ValueKey('sheet-body-canvas'),
                    page: widget.page,
                    canvasWidth: widget.page.width,
                    canvasHeight: widget.page.height,
                    scaleX: 1,
                    scaleY: 1,
                    highlights: widget.highlights,
                    backgroundColor: Colors.white,
                    clipBehavior: Clip.hardEdge,
                    onSelectionStart: widget.onSelectionStart,
                    onSelectionUpdate: widget.onSelectionUpdate,
                    onSelectionEnd: widget.onSelectionEnd,
                  ),
                ),
              ),
            ),
          ),
        ),
        if (frozenColumnCount > 0 || frozenRowCount > 0)
          IgnorePointer(
            child: _FrozenPaneOverlay(
              key: const ValueKey('sheet-frozen-overlay'),
              page: widget.page,
              highlights: widget.highlights,
              horizontalOffset: _horizontalBodyController.hasClients
                  ? _horizontalBodyController.offset
                  : 0,
              verticalOffset: _verticalBodyController.hasClients
                  ? _verticalBodyController.offset
                  : 0,
              frozenWidth: frozenWidth,
              frozenHeight: frozenHeight,
              freezeColumns: frozenColumnCount > 0,
              freezeRows: frozenRowCount > 0,
            ),
          ),
      ],
    );
  }
}

class _FrozenPaneOverlay extends StatelessWidget {
  const _FrozenPaneOverlay({
    super.key,
    required this.page,
    required this.highlights,
    required this.horizontalOffset,
    required this.verticalOffset,
    required this.frozenWidth,
    required this.frozenHeight,
    required this.freezeColumns,
    required this.freezeRows,
  });

  final PageRenderModel page;
  final List<Rect> highlights;
  final double horizontalOffset;
  final double verticalOffset;
  final double frozenWidth;
  final double frozenHeight;
  final bool freezeColumns;
  final bool freezeRows;

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        if (freezeRows && frozenHeight > 0)
          Positioned(
            left: freezeColumns ? frozenWidth : 0,
            top: 0,
            right: 0,
            height: frozenHeight,
            child: ClipRect(
              child: Transform.translate(
                offset: Offset(-horizontalOffset, 0),
                child: RenderNodeCanvas(
                  page: page,
                  canvasWidth: page.width,
                  canvasHeight: page.height,
                  scaleX: 1,
                  scaleY: 1,
                  highlights: highlights,
                  backgroundColor: Colors.transparent,
                ),
              ),
            ),
          ),
        if (freezeColumns && frozenWidth > 0)
          Positioned(
            left: 0,
            top: freezeRows ? frozenHeight : 0,
            bottom: 0,
            width: frozenWidth,
            child: ClipRect(
              child: Transform.translate(
                offset: Offset(0, -verticalOffset),
                child: RenderNodeCanvas(
                  page: page,
                  canvasWidth: page.width,
                  canvasHeight: page.height,
                  scaleX: 1,
                  scaleY: 1,
                  highlights: highlights,
                  backgroundColor: Colors.transparent,
                ),
              ),
            ),
          ),
        if (freezeColumns && freezeRows && frozenWidth > 0 && frozenHeight > 0)
          Positioned(
            left: 0,
            top: 0,
            width: frozenWidth,
            height: frozenHeight,
            child: ClipRect(
              child: RenderNodeCanvas(
                page: page,
                canvasWidth: page.width,
                canvasHeight: page.height,
                scaleX: 1,
                scaleY: 1,
                highlights: highlights,
                backgroundColor: Colors.transparent,
              ),
            ),
          ),
        if (freezeColumns && frozenWidth > 0)
          Positioned(
            left: frozenWidth - 1,
            top: 0,
            bottom: 0,
            child: Container(
              key: const ValueKey('sheet-frozen-vertical-divider'),
              width: 1,
              color: const Color(0xFF9FB1C1),
            ),
          ),
        if (freezeRows && frozenHeight > 0)
          Positioned(
            left: 0,
            top: frozenHeight - 1,
            right: 0,
            child: Container(
              key: const ValueKey('sheet-frozen-horizontal-divider'),
              height: 1,
              color: const Color(0xFF9FB1C1),
            ),
          ),
      ],
    );
  }
}

class _SheetHeaderCell extends StatelessWidget {
  const _SheetHeaderCell({
    required this.label,
    required this.alignment,
    this.padding = const EdgeInsets.symmetric(horizontal: 8),
  });

  final String label;
  final Alignment alignment;
  final EdgeInsetsGeometry padding;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: const BoxDecoration(
        border: Border(
          right: BorderSide(color: _SheetViewportState._headerBorder),
          bottom: BorderSide(color: _SheetViewportState._headerBorder),
        ),
      ),
      alignment: alignment,
      padding: padding,
      child: Text(
        label,
        overflow: TextOverflow.ellipsis,
        style: Theme.of(context).textTheme.labelMedium?.copyWith(
          fontWeight: FontWeight.w600,
          color: const Color(0xFF425466),
        ),
      ),
    );
  }
}

class _SheetViewportMetrics {
  const _SheetViewportMetrics({
    required this.columns,
    required this.rows,
    required this.visibleRows,
    required this.visibleColumns,
  });

  final List<_SheetAxisSegment> columns;
  final List<_SheetAxisSegment> rows;
  final List<int> visibleRows;
  final List<int> visibleColumns;

  factory _SheetViewportMetrics.fromPage(PageRenderModel page) {
    final boxes = page.nodes.whereType<BoxRenderNodeModel>().toList(
      growable: false,
    );
    final viewport = page.sheetViewport;
    final columnSegments = boxes.isEmpty
        ? [_SheetAxisSegment(start: 0, end: math.max(page.width, 1))]
        : _extractSegments(
            boxes.map((box) => box.bounds.x).toList(growable: false),
            boxes
                .map((box) => box.bounds.x + box.bounds.width)
                .toList(growable: false),
            page.width,
          );
    final rowSegments = boxes.isEmpty
        ? [_SheetAxisSegment(start: 0, end: math.max(page.height, 1))]
        : _extractSegments(
            boxes.map((box) => box.bounds.y).toList(growable: false),
            boxes
                .map((box) => box.bounds.y + box.bounds.height)
                .toList(growable: false),
            page.height,
          );

    return _SheetViewportMetrics(
      columns: columnSegments,
      rows: rowSegments,
      visibleRows: _visibleIndexes(
        explicit: viewport?.visibleRows,
        start: viewport?.window.startRow ?? 1,
        count: rowSegments.length,
      ),
      visibleColumns: _visibleIndexes(
        explicit: viewport?.visibleColumns,
        start: viewport?.window.startColumn ?? 1,
        count: columnSegments.length,
      ),
    );
  }

  double extentForLeadingColumns(int count) {
    return columns
        .take(count.clamp(0, columns.length))
        .fold(0.0, (sum, segment) => sum + segment.extent);
  }

  double extentForLeadingRows(int count) {
    return rows
        .take(count.clamp(0, rows.length))
        .fold(0.0, (sum, segment) => sum + segment.extent);
  }

  static List<_SheetAxisSegment> _extractSegments(
    List<double> starts,
    List<double> ends,
    double maxExtent,
  ) {
    final edges = <double>{0};
    for (final start in starts) {
      edges.add(_normalizeEdge(start));
    }
    for (final end in ends) {
      edges.add(_normalizeEdge(end));
    }
    edges.add(_normalizeEdge(maxExtent));

    final sortedEdges = edges.toList()..sort();
    final segments = <_SheetAxisSegment>[];
    for (var index = 0; index < sortedEdges.length - 1; index++) {
      final start = sortedEdges[index];
      final end = sortedEdges[index + 1];
      if (end - start <= 0.5) {
        continue;
      }
      segments.add(_SheetAxisSegment(start: start, end: end));
    }

    return segments.isEmpty
        ? [_SheetAxisSegment(start: 0, end: math.max(maxExtent, 1))]
        : segments;
  }

  static double _normalizeEdge(double value) {
    return (value * 100).roundToDouble() / 100;
  }

  static List<int> _visibleIndexes({
    required List<int>? explicit,
    required int start,
    required int count,
  }) {
    if (explicit != null && explicit.length == count) {
      return explicit;
    }
    return List<int>.generate(count, (index) => start + index, growable: false);
  }
}

class _SheetAxisSegment {
  const _SheetAxisSegment({required this.start, required this.end});

  final double start;
  final double end;

  double get extent => end - start;
}

bool _sameWindowBounds(SheetBoundsModel left, SheetBoundsModel right) {
  return left.startRow == right.startRow &&
      left.endRow == right.endRow &&
      left.startColumn == right.startColumn &&
      left.endColumn == right.endColumn;
}

String _columnLabel(int index) {
  var current = index;
  final buffer = StringBuffer();
  while (current > 0) {
    final remainder = (current - 1) % 26;
    buffer.writeCharCode(65 + remainder);
    current = (current - 1) ~/ 26;
  }
  return buffer.toString().split('').reversed.join();
}

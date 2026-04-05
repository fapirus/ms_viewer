import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import 'render_node_canvas.dart';

class SheetViewport extends StatefulWidget {
  const SheetViewport({
    super.key,
    required this.page,
    this.highlights = const [],
    this.focusRect,
    this.onCellTap,
    this.onSelectionStart,
    this.onSelectionUpdate,
    this.onSelectionEnd,
    this.onWindowRequest,
  });

  final PageRenderModel page;
  final List<Rect> highlights;
  final Rect? focusRect;
  final ValueChanged<Offset>? onCellTap;
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
  static const double _leadingEdgeThresholdPixels = 96;
  static const int _minimumVisibleRows = 40;
  static const int _minimumVisibleColumns = 16;
  static const int _initialRowOverscan = 12;
  static const int _initialColumnOverscan = 4;
  static const double _targetRowPixels = 24;
  static const double _targetColumnPixels = 72;
  static const double _viewportRowOverscanPixels = 240;
  static const double _viewportColumnOverscanPixels = 320;
  static const int _excelMaxRows = 1048576;
  static const int _excelMaxColumns = 16384;

  final ScrollController _horizontalBodyController = ScrollController();
  final ScrollController _verticalBodyController = ScrollController();
  final ScrollController _horizontalHeaderController = ScrollController();
  final ScrollController _verticalHeaderController = ScrollController();

  bool _syncingHorizontal = false;
  bool _syncingVertical = false;
  bool _windowRequestInFlight = false;
  int _pendingPrependedColumns = 0;
  int _pendingPrependedRows = 0;
  _SheetViewportMetrics? _latestMetrics;
  bool _initialWindowExpansionScheduled = false;
  Size? _lastBodyViewportSize;
  Rect? _lastEnsuredFocusRect;

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
      _lastEnsuredFocusRect = null;
      final newMetrics = _SheetViewportMetrics.fromPage(widget.page);
      final horizontalAdjustment = _pendingPrependedColumns > 0
          ? newMetrics.extentForLeadingColumns(_pendingPrependedColumns)
          : 0.0;
      final verticalAdjustment = _pendingPrependedRows > 0
          ? newMetrics.extentForLeadingRows(_pendingPrependedRows)
          : 0.0;
      _pendingPrependedColumns = 0;
      _pendingPrependedRows = 0;
      _windowRequestInFlight = false;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) {
          return;
        }
        if (horizontalAdjustment > 0 && _horizontalBodyController.hasClients) {
          _horizontalBodyController.jumpTo(
            (_horizontalBodyController.offset + horizontalAdjustment).clamp(
              0.0,
              _horizontalBodyController.position.maxScrollExtent,
            ),
          );
        }
        if (verticalAdjustment > 0 && _verticalBodyController.hasClients) {
          _verticalBodyController.jumpTo(
            (_verticalBodyController.offset + verticalAdjustment).clamp(
              0.0,
              _verticalBodyController.position.maxScrollExtent,
            ),
          );
        }
      });
    }
    if (widget.focusRect != null &&
        (oldWidget.focusRect == null || oldWidget.focusRect != widget.focusRect)) {
      _lastEnsuredFocusRect = null;
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
    final rowCount = currentWindow.endRow - currentWindow.startRow + 1;
    final columnCount = currentWindow.endColumn - currentWindow.startColumn + 1;
    var nextStartRow = currentWindow.startRow;
    var nextEndRow = currentWindow.endRow;
    var nextStartColumn = currentWindow.startColumn;
    var nextEndColumn = currentWindow.endColumn;
    var prependedColumns = 0;
    var prependedRows = 0;

    if (_horizontalBodyController.hasClients &&
        _horizontalBodyController.position.maxScrollExtent > 0 &&
        _horizontalBodyController.offset >=
            _horizontalBodyController.position.maxScrollExtent *
                _requestThreshold &&
        currentWindow.endColumn < _excelMaxColumns) {
      final shift = math.max(1, (columnCount * _windowShiftRatio).round());
      nextEndColumn = math.min(currentWindow.endColumn + shift, _excelMaxColumns);
    } else if (_horizontalBodyController.hasClients &&
        currentWindow.startColumn > 1 &&
        _horizontalBodyController.offset <= _leadingEdgeThresholdPixels) {
      final shift = math.max(1, (columnCount * _windowShiftRatio).round());
      nextStartColumn = math.max(1, currentWindow.startColumn - shift);
      prependedColumns = currentWindow.startColumn - nextStartColumn;
    }

    if (_verticalBodyController.hasClients &&
        _verticalBodyController.position.maxScrollExtent > 0 &&
        _verticalBodyController.offset >=
            _verticalBodyController.position.maxScrollExtent *
                _requestThreshold &&
        currentWindow.endRow < _excelMaxRows) {
      final shift = math.max(1, (rowCount * _windowShiftRatio).round());
      nextEndRow = math.min(currentWindow.endRow + shift, _excelMaxRows);
    } else if (_verticalBodyController.hasClients &&
        currentWindow.startRow > 1 &&
        _verticalBodyController.offset <= _leadingEdgeThresholdPixels) {
      final shift = math.max(1, (rowCount * _windowShiftRatio).round());
      nextStartRow = math.max(1, currentWindow.startRow - shift);
      prependedRows = currentWindow.startRow - nextStartRow;
    }

    if (nextStartRow == currentWindow.startRow &&
        nextEndRow == currentWindow.endRow &&
        nextStartColumn == currentWindow.startColumn &&
        nextEndColumn == currentWindow.endColumn) {
      return;
    }

    _windowRequestInFlight = true;
    _pendingPrependedColumns = prependedColumns;
    _pendingPrependedRows = prependedRows;
    unawaited(
      callback(
        SheetWindow(
          startRow: nextStartRow,
          endRow: nextEndRow,
          startColumn: nextStartColumn,
          endColumn: nextEndColumn,
        ),
      ).catchError((_) {
        _windowRequestInFlight = false;
        _pendingPrependedColumns = 0;
        _pendingPrependedRows = 0;
      }),
    );
  }

  void _handlePointerSignal(PointerSignalEvent event) {
    if (event is! PointerScrollEvent) {
      return;
    }

    GestureBinding.instance.pointerSignalResolver.register(event, (
      PointerSignalEvent resolvedEvent,
    ) {
      if (resolvedEvent is! PointerScrollEvent) {
        return;
      }

      var didScroll = false;
      if (_horizontalBodyController.hasClients &&
          resolvedEvent.scrollDelta.dx != 0) {
        final nextHorizontalOffset = (_horizontalBodyController.offset +
                resolvedEvent.scrollDelta.dx)
            .clamp(0.0, _horizontalBodyController.position.maxScrollExtent);
        if ((nextHorizontalOffset - _horizontalBodyController.offset).abs() >
            0.5) {
          _horizontalBodyController.jumpTo(nextHorizontalOffset);
          didScroll = true;
        }
      }

      if (_verticalBodyController.hasClients &&
          resolvedEvent.scrollDelta.dy != 0) {
        final nextVerticalOffset = (_verticalBodyController.offset +
                resolvedEvent.scrollDelta.dy)
            .clamp(0.0, _verticalBodyController.position.maxScrollExtent);
        if ((nextVerticalOffset - _verticalBodyController.offset).abs() > 0.5) {
          _verticalBodyController.jumpTo(nextVerticalOffset);
          didScroll = true;
        }
      }

      if (didScroll && mounted) {
        _maybeRequestWindow();
        setState(() {});
      }
    });
  }

  void _ensureRectVisible(Rect rect) {
    final viewportSize = _lastBodyViewportSize;
    if (viewportSize == null ||
        !_horizontalBodyController.hasClients ||
        !_verticalBodyController.hasClients) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) {
          return;
        }
        _ensureRectVisible(rect);
      });
      return;
    }

    var nextHorizontalOffset = _horizontalBodyController.offset;
    var nextVerticalOffset = _verticalBodyController.offset;

    if (rect.left < nextHorizontalOffset) {
      nextHorizontalOffset = rect.left;
    } else if (rect.right > nextHorizontalOffset + viewportSize.width) {
      nextHorizontalOffset = rect.right - viewportSize.width;
    }

    if (rect.top < nextVerticalOffset) {
      nextVerticalOffset = rect.top;
    } else if (rect.bottom > nextVerticalOffset + viewportSize.height) {
      nextVerticalOffset = rect.bottom - viewportSize.height;
    }

    nextHorizontalOffset = nextHorizontalOffset.clamp(
      0.0,
      _horizontalBodyController.position.maxScrollExtent,
    );
    nextVerticalOffset = nextVerticalOffset.clamp(
      0.0,
      _verticalBodyController.position.maxScrollExtent,
    );

    if ((nextHorizontalOffset - _horizontalBodyController.offset).abs() > 0.5) {
      _horizontalBodyController.jumpTo(nextHorizontalOffset);
      if (_horizontalHeaderController.hasClients) {
        _horizontalHeaderController.jumpTo(
          nextHorizontalOffset.clamp(
            0.0,
            _horizontalHeaderController.position.maxScrollExtent,
          ),
        );
      }
    }
    if ((nextVerticalOffset - _verticalBodyController.offset).abs() > 0.5) {
      _verticalBodyController.jumpTo(nextVerticalOffset);
      if (_verticalHeaderController.hasClients) {
        _verticalHeaderController.jumpTo(
          nextVerticalOffset.clamp(
            0.0,
            _verticalHeaderController.position.maxScrollExtent,
          ),
        );
      }
    }
    if (mounted) {
      setState(() {});
    }
  }

  @override
  Widget build(BuildContext context) {
    final metrics = _SheetViewportMetrics.fromPage(widget.page);
    _latestMetrics = metrics;

    return LayoutBuilder(
      builder: (context, constraints) {
        _scheduleMinimumWindowExpansion(constraints.biggest, metrics);
        _scheduleFocusFollow();
        return Listener(
          behavior: HitTestBehavior.opaque,
          onPointerSignal: _handlePointerSignal,
          child: DecoratedBox(
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
          ),
        );
      },
    );
  }

  void _scheduleFocusFollow() {
    final focusRect = widget.focusRect;
    if (focusRect == null || focusRect == _lastEnsuredFocusRect) {
      return;
    }
    _lastEnsuredFocusRect = focusRect;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) {
        return;
      }
      final currentFocusRect = widget.focusRect;
      if (currentFocusRect == null) {
        return;
      }
      _ensureRectVisible(currentFocusRect);
    });
  }

  void _scheduleMinimumWindowExpansion(
    Size viewportSize,
    _SheetViewportMetrics metrics,
  ) {
    final callback = widget.onWindowRequest;
    final viewport = widget.page.sheetViewport;
    if (callback == null || viewport == null) {
      return;
    }

    final currentWindow = viewport.window;
    final bodyViewportWidth =
        math.max(0.0, viewportSize.width - _cornerExtent - 1);
    final bodyViewportHeight =
        math.max(0.0, viewportSize.height - _headerExtent - 1);
    final bodyViewportSize = Size(bodyViewportWidth, bodyViewportHeight);
    if (_lastBodyViewportSize == null ||
        bodyViewportSize.width > _lastBodyViewportSize!.width + 24 ||
        bodyViewportSize.height > _lastBodyViewportSize!.height + 24) {
      _initialWindowExpansionScheduled = false;
    }
    _lastBodyViewportSize = bodyViewportSize;
    if (_initialWindowExpansionScheduled) {
      return;
    }
    final averageColumnExtent = metrics.columns.isEmpty
        ? _targetColumnPixels
        : (metrics.columns.fold<double>(
                  0,
                  (sum, segment) => sum + segment.extent,
                ) /
                metrics.columns.length)
            .clamp(1.0, double.infinity);
    final averageRowExtent = metrics.rows.isEmpty
        ? _targetRowPixels
        : (metrics.rows.fold<double>(
                  0,
                  (sum, segment) => sum + segment.extent,
                ) /
                metrics.rows.length)
            .clamp(1.0, double.infinity);
    final desiredRowCount = math.max(
      _minimumVisibleRows,
      math.max(
            ((viewportSize.height - _headerExtent) / _targetRowPixels).ceil() +
                _initialRowOverscan,
            ((bodyViewportHeight + _viewportRowOverscanPixels) /
                    averageRowExtent)
                .ceil(),
          ),
    );
    final desiredColumnCount = math.max(
      _minimumVisibleColumns,
      math.max(
            ((viewportSize.width - _cornerExtent) / _targetColumnPixels).ceil() +
                _initialColumnOverscan,
            ((bodyViewportWidth + _viewportColumnOverscanPixels) /
                    averageColumnExtent)
                .ceil(),
          ),
    );
    final currentRowCount = currentWindow.endRow - currentWindow.startRow + 1;
    final currentColumnCount =
        currentWindow.endColumn - currentWindow.startColumn + 1;
    final targetEndRow = math.min(
      _excelMaxRows,
      currentWindow.startRow + desiredRowCount - 1,
    );
    final targetEndColumn = math.min(
      _excelMaxColumns,
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

    return LayoutBuilder(
      builder: (context, constraints) {
        final viewportSize = Size(
          constraints.maxWidth,
          constraints.maxHeight,
        );
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
                      child: SheetRenderCanvas(
                        key: const ValueKey('sheet-body-canvas'),
                        page: widget.page,
                        canvasWidth: widget.page.width,
                        canvasHeight: widget.page.height,
                        viewportOffset: Offset(
                          _horizontalBodyController.hasClients
                              ? _horizontalBodyController.offset
                              : 0,
                          _verticalBodyController.hasClients
                              ? _verticalBodyController.offset
                              : 0,
                        ),
                        viewportSize: viewportSize,
                        highlights: widget.highlights,
                        backgroundColor: Colors.white,
                        clipBehavior: Clip.hardEdge,
                        onCellTap: widget.onCellTap,
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
                  viewportSize: viewportSize,
                  frozenWidth: frozenWidth,
                  frozenHeight: frozenHeight,
                  freezeColumns: frozenColumnCount > 0,
                  freezeRows: frozenRowCount > 0,
                ),
              ),
          ],
        );
      },
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
    required this.viewportSize,
    required this.frozenWidth,
    required this.frozenHeight,
    required this.freezeColumns,
    required this.freezeRows,
  });

  final PageRenderModel page;
  final List<Rect> highlights;
  final double horizontalOffset;
  final double verticalOffset;
  final Size viewportSize;
  final double frozenWidth;
  final double frozenHeight;
  final bool freezeColumns;
  final bool freezeRows;

  @override
  Widget build(BuildContext context) {
    final splitPage = _filterSheetPageNodes(
      page,
      (node) => !_isFrozenBoundaryCrossingTextNode(
            node,
            frozenWidth: frozenWidth,
            frozenHeight: frozenHeight,
            freezeColumns: freezeColumns,
            freezeRows: freezeRows,
          ),
    );
    final topCrossingTextPage = _filterSheetPageNodes(
      page,
      (node) => _isFrozenBoundaryCrossingTextNode(
        node,
        frozenWidth: frozenWidth,
        frozenHeight: frozenHeight,
        freezeColumns: freezeColumns,
        freezeRows: freezeRows,
      ),
    );
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
                child: SheetRenderCanvas(
                  page: splitPage,
                  canvasWidth: page.width,
                  canvasHeight: page.height,
                  viewportOffset: Offset(horizontalOffset, 0),
                  viewportSize: Size(viewportSize.width, frozenHeight),
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
                child: SheetRenderCanvas(
                  page: splitPage,
                  canvasWidth: page.width,
                  canvasHeight: page.height,
                  viewportOffset: Offset(0, verticalOffset),
                  viewportSize: Size(frozenWidth, viewportSize.height),
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
                child: SheetRenderCanvas(
                  page: splitPage,
                  canvasWidth: page.width,
                  canvasHeight: page.height,
                  viewportOffset: Offset.zero,
                  viewportSize: Size(frozenWidth, frozenHeight),
                  highlights: highlights,
                  backgroundColor: Colors.transparent,
                ),
              ),
            ),
        if (topCrossingTextPage.nodes.isNotEmpty && freezeRows && frozenHeight > 0)
          Positioned.fill(
            child: IgnorePointer(
              child: ClipRect(
                child: SheetRenderCanvas(
                  page: topCrossingTextPage,
                  canvasWidth: page.width,
                  canvasHeight: page.height,
                  viewportOffset: Offset.zero,
                  viewportSize: Size(viewportSize.width, frozenHeight),
                  highlights: highlights,
                  backgroundColor: Colors.transparent,
                ),
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

PageRenderModel _filterSheetPageNodes(
  PageRenderModel page,
  bool Function(RenderNodeModel node) predicate,
) {
  return PageRenderModel(
    pageIndex: page.pageIndex,
    width: page.width,
    height: page.height,
    nodes: page.nodes.where(predicate).toList(growable: false),
    selectionAnchors: const [],
    sheetViewport: page.sheetViewport,
    sheetCells: const [],
  );
}

bool _isFrozenBoundaryCrossingTextNode(
  RenderNodeModel node, {
  required double frozenWidth,
  required double frozenHeight,
  required bool freezeColumns,
  required bool freezeRows,
}) {
  if (node is! TextRenderNodeModel) {
    return false;
  }

  final bounds = Rect.fromLTWH(
      node.bounds.x,
      node.bounds.y,
      node.bounds.width,
      node.bounds.height,
    );

  final crossesFrozenColumn = freezeColumns &&
      freezeRows &&
      bounds.left < frozenWidth &&
      bounds.right > frozenWidth &&
      bounds.top < frozenHeight;
  final crossesFrozenRow = freezeRows &&
      freezeColumns &&
      bounds.top < frozenHeight &&
      bounds.bottom > frozenHeight &&
      bounds.left < frozenWidth;

  return crossesFrozenColumn || crossesFrozenRow;
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
    final sheetCells = page.sheetCells;
    final viewport = page.sheetViewport;
    final columnSegments = sheetCells.isNotEmpty
        ? _extractSegments(
            sheetCells.map((cell) => cell.bounds.x).toList(growable: false),
            sheetCells
                .map((cell) => cell.bounds.x + cell.bounds.width)
                .toList(growable: false),
            page.width,
          )
        : boxes.isEmpty
        ? [_SheetAxisSegment(start: 0, end: math.max(page.width, 1))]
        : _extractSegments(
            boxes.map((box) => box.bounds.x).toList(growable: false),
            boxes
                .map((box) => box.bounds.x + box.bounds.width)
                .toList(growable: false),
            page.width,
          );
    final rowSegments = sheetCells.isNotEmpty
        ? _extractSegments(
            sheetCells.map((cell) => cell.bounds.y).toList(growable: false),
            sheetCells
                .map((cell) => cell.bounds.y + cell.bounds.height)
                .toList(growable: false),
            page.height,
          )
        : boxes.isEmpty
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

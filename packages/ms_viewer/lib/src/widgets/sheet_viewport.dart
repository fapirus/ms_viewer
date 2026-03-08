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
  });

  final PageRenderModel page;
  final List<Rect> highlights;
  final ValueChanged<Offset>? onSelectionStart;
  final ValueChanged<Offset>? onSelectionUpdate;
  final VoidCallback? onSelectionEnd;

  @override
  State<SheetViewport> createState() => _SheetViewportState();
}

class _SheetViewportState extends State<SheetViewport> {
  static const double _cornerExtent = 56;
  static const double _headerExtent = 36;
  static const Color _headerBackground = Color(0xFFF3F6FA);
  static const Color _headerBorder = Color(0xFFD0D7DE);

  final ScrollController _horizontalBodyController = ScrollController();
  final ScrollController _verticalBodyController = ScrollController();
  final ScrollController _horizontalHeaderController = ScrollController();
  final ScrollController _verticalHeaderController = ScrollController();

  bool _syncingHorizontal = false;
  bool _syncingVertical = false;

  @override
  void initState() {
    super.initState();
    _horizontalBodyController.addListener(_syncHorizontalOffset);
    _verticalBodyController.addListener(_syncVerticalOffset);
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
  }

  @override
  Widget build(BuildContext context) {
    final metrics = _SheetViewportMetrics.fromPage(widget.page);

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
                  Expanded(child: _buildScrollableBody()),
                ],
              ),
            ),
          ],
        ),
      ),
    );
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
                    label: _columnLabel(index + 1),
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
                    label: '${index + 1}',
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

  Widget _buildScrollableBody() {
    return KeyedSubtree(
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
  const _SheetViewportMetrics({required this.columns, required this.rows});

  final List<_SheetAxisSegment> columns;
  final List<_SheetAxisSegment> rows;

  factory _SheetViewportMetrics.fromPage(PageRenderModel page) {
    final boxes = page.nodes.whereType<BoxRenderNodeModel>().toList(
      growable: false,
    );
    if (boxes.isEmpty) {
      return _SheetViewportMetrics(
        columns: [_SheetAxisSegment(start: 0, end: math.max(page.width, 1))],
        rows: [_SheetAxisSegment(start: 0, end: math.max(page.height, 1))],
      );
    }

    return _SheetViewportMetrics(
      columns: _extractSegments(
        boxes.map((box) => box.bounds.x).toList(growable: false),
        boxes
            .map((box) => box.bounds.x + box.bounds.width)
            .toList(growable: false),
        page.width,
      ),
      rows: _extractSegments(
        boxes.map((box) => box.bounds.y).toList(growable: false),
        boxes
            .map((box) => box.bounds.y + box.bounds.height)
            .toList(growable: false),
        page.height,
      ),
    );
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
}

class _SheetAxisSegment {
  const _SheetAxisSegment({required this.start, required this.end});

  final double start;
  final double end;

  double get extent => end - start;
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

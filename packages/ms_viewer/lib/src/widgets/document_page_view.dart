import 'package:flutter/material.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import 'selection_highlight_overlay.dart';

class DocumentPageView extends StatelessWidget {
  const DocumentPageView({
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
  Widget build(BuildContext context) {
    return AspectRatio(
      aspectRatio: page.width / page.height,
      child: LayoutBuilder(
        builder: (context, constraints) {
          final size = constraints.biggest;
          final scaleX = size.width / page.width;
          final scaleY = size.height / page.height;
          final transformedHighlights = highlights
              .map(
                (rect) => Rect.fromLTWH(
                  rect.left * scaleX,
                  rect.top * scaleY,
                  rect.width * scaleX,
                  rect.height * scaleY,
                ),
              )
              .toList(growable: false);

          return GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanStart: onSelectionStart == null
                ? null
                : (details) => onSelectionStart!(
                      _toPageOffset(details.localPosition, scaleX, scaleY),
                    ),
            onPanUpdate: onSelectionUpdate == null
                ? null
                : (details) => onSelectionUpdate!(
                      _toPageOffset(details.localPosition, scaleX, scaleY),
                    ),
            onPanEnd: onSelectionEnd == null ? null : (_) => onSelectionEnd!(),
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: Colors.white,
                border: Border.all(color: const Color(0xFFD7D7D7)),
                boxShadow: const [
                  BoxShadow(
                    color: Color(0x12000000),
                    blurRadius: 12,
                    offset: Offset(0, 4),
                  ),
                ],
              ),
              child: Stack(
                children: [
                  CustomPaint(
                    size: Size.infinite,
                    painter: _PageRenderPainter(page: page),
                  ),
                  IgnorePointer(
                    child: SelectionHighlightOverlay(
                      highlights: transformedHighlights,
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}

Offset _toPageOffset(Offset localPosition, double scaleX, double scaleY) {
  return Offset(localPosition.dx / scaleX, localPosition.dy / scaleY);
}

class _PageRenderPainter extends CustomPainter {
  _PageRenderPainter({required this.page});

  final PageRenderModel page;

  @override
  void paint(Canvas canvas, Size size) {
    final scaleX = size.width / page.width;
    final scaleY = size.height / page.height;

    for (final node in page.nodes) {
      switch (node) {
        case TextRenderNodeModel():
          final painter = TextPainter(
            text: TextSpan(
              text: node.text,
              style: TextStyle(
                fontSize: node.style.fontSize * scaleY,
                fontWeight: node.style.bold ? FontWeight.w700 : FontWeight.w400,
                fontStyle: node.style.italic ? FontStyle.italic : FontStyle.normal,
                color: _parseColor(node.style.colorHex),
              ),
            ),
            textDirection: TextDirection.ltr,
            maxLines: 1,
          )..layout(maxWidth: node.bounds.width * scaleX);
          painter.paint(canvas, Offset(node.bounds.x * scaleX, node.bounds.y * scaleY));
        case BoxRenderNodeModel():
          final rect = Rect.fromLTWH(
            node.bounds.x * scaleX,
            node.bounds.y * scaleY,
            node.bounds.width * scaleX,
            node.bounds.height * scaleY,
          );
          final paint = Paint()
            ..style = PaintingStyle.fill
            ..color = _parseColor(node.fillColorHex) ?? Colors.transparent;
          canvas.drawRect(rect, paint);
          if (node.strokeWidth > 0) {
            canvas.drawRect(
              rect,
              Paint()
                ..style = PaintingStyle.stroke
                ..strokeWidth = node.strokeWidth * scaleX
                ..color = _parseColor(node.strokeColorHex) ?? Colors.black,
            );
          }
        case ImageRenderNodeModel():
          final rect = Rect.fromLTWH(
            node.bounds.x * scaleX,
            node.bounds.y * scaleY,
            node.bounds.width * scaleX,
            node.bounds.height * scaleY,
          );
          canvas.drawRect(
            rect,
            Paint()..color = const Color(0xFFE9EEF7),
          );
          final painter = TextPainter(
            text: TextSpan(
              text: node.description ?? node.resourceId,
              style: const TextStyle(
                color: Color(0xFF42526B),
                fontSize: 10,
              ),
            ),
            textDirection: TextDirection.ltr,
            maxLines: 2,
            ellipsis: '…',
          )..layout(maxWidth: rect.width - 8);
          painter.paint(canvas, Offset(rect.left + 4, rect.top + 4));
      }
    }
  }

  @override
  bool shouldRepaint(covariant _PageRenderPainter oldDelegate) {
    return oldDelegate.page != page;
  }
}

Color? _parseColor(String? hex) {
  if (hex == null) {
    return null;
  }
  final normalized = hex.replaceFirst('#', '');
  if (normalized.length != 6) {
    return null;
  }
  return Color(int.parse('FF$normalized', radix: 16));
}

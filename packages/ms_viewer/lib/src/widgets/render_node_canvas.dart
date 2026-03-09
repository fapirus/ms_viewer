import 'dart:convert';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart'
    show TargetPlatform, defaultTargetPlatform, visibleForTesting;
import 'package:flutter/material.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import '../painting/font_fallback_policy.dart';
import 'selection_highlight_overlay.dart';

class RenderNodeCanvas extends StatelessWidget {
  const RenderNodeCanvas({
    super.key,
    required this.page,
    required this.canvasWidth,
    required this.canvasHeight,
    required this.scaleX,
    required this.scaleY,
    this.highlights = const [],
    this.backgroundColor = Colors.transparent,
    this.clipBehavior = Clip.none,
    this.onSelectionStart,
    this.onSelectionUpdate,
    this.onSelectionEnd,
  });

  final PageRenderModel page;
  final double canvasWidth;
  final double canvasHeight;
  final double scaleX;
  final double scaleY;
  final List<Rect> highlights;
  final Color backgroundColor;
  final Clip clipBehavior;
  final ValueChanged<Offset>? onSelectionStart;
  final ValueChanged<Offset>? onSelectionUpdate;
  final VoidCallback? onSelectionEnd;

  @override
  Widget build(BuildContext context) {
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

    return SizedBox(
      width: canvasWidth,
      height: canvasHeight,
      child: GestureDetector(
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
        child: ColoredBox(
          color: backgroundColor,
          child: ClipRect(
            clipBehavior: clipBehavior,
            child: Stack(
              clipBehavior: clipBehavior,
              children: [
                ...buildRenderLayers(
                  page: page,
                  platform: viewerPlatformForTargetPlatform(
                    defaultTargetPlatform,
                  ),
                  scaleX: scaleX,
                  scaleY: scaleY,
                ),
                IgnorePointer(
                  child: SelectionHighlightOverlay(
                    highlights: transformedHighlights,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class SheetRenderCanvas extends StatelessWidget {
  const SheetRenderCanvas({
    super.key,
    required this.page,
    required this.canvasWidth,
    required this.canvasHeight,
    required this.viewportOffset,
    required this.viewportSize,
    this.highlights = const [],
    this.backgroundColor = Colors.transparent,
    this.clipBehavior = Clip.none,
    this.onCellTap,
    this.onSelectionStart,
    this.onSelectionUpdate,
    this.onSelectionEnd,
  });

  final PageRenderModel page;
  final double canvasWidth;
  final double canvasHeight;
  final Offset viewportOffset;
  final Size viewportSize;
  final List<Rect> highlights;
  final Color backgroundColor;
  final Clip clipBehavior;
  final ValueChanged<Offset>? onCellTap;
  final ValueChanged<Offset>? onSelectionStart;
  final ValueChanged<Offset>? onSelectionUpdate;
  final VoidCallback? onSelectionEnd;

  @override
  Widget build(BuildContext context) {
    final visibleRect = Rect.fromLTWH(
      viewportOffset.dx,
      viewportOffset.dy,
      viewportSize.width,
      viewportSize.height,
    );
    final transformedHighlights = highlights
        .where((rect) => rect.overlaps(visibleRect))
        .map(
          (rect) => Rect.fromLTWH(
            rect.left,
            rect.top,
            rect.width,
            rect.height,
          ),
        )
        .toList(growable: false);
    final platform = viewerPlatformForTargetPlatform(defaultTargetPlatform);
    final visibleImageNodes = <(int, ImageRenderNodeModel)>[];
    for (var index = 0; index < page.nodes.length; index++) {
      final node = page.nodes[index];
      if (node case ImageRenderNodeModel()) {
        final bounds = Rect.fromLTWH(
          node.bounds.x,
          node.bounds.y,
          node.bounds.width,
          node.bounds.height,
        );
        if (bounds.overlaps(visibleRect)) {
          visibleImageNodes.add((index, node));
        }
      }
    }

    return SizedBox(
      width: canvasWidth,
      height: canvasHeight,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapUp: onCellTap == null
            ? null
            : (details) => onCellTap!(details.localPosition),
        onPanStart: onSelectionStart == null
            ? null
            : (details) => onSelectionStart!(details.localPosition),
        onPanUpdate: onSelectionUpdate == null
            ? null
            : (details) => onSelectionUpdate!(details.localPosition),
        onPanEnd: onSelectionEnd == null ? null : (_) => onSelectionEnd!(),
        child: RepaintBoundary(
          child: ColoredBox(
            color: backgroundColor,
            child: ClipRect(
              clipBehavior: clipBehavior,
              child: Stack(
                clipBehavior: clipBehavior,
                children: [
                  CustomPaint(
                    size: Size(canvasWidth, canvasHeight),
                    painter: _SheetNodePainter(
                      page: page,
                      platform: platform,
                      visibleRect: visibleRect,
                    ),
                  ),
                  for (final (index, node) in visibleImageNodes)
                    Positioned(
                      key: ValueKey('sheet-image-node-$index'),
                      left: node.bounds.x,
                      top: node.bounds.y,
                      width: node.bounds.width,
                      height: node.bounds.height,
                      child: IgnorePointer(
                        child: _ImageNodeLayer(
                          node: node,
                          width: node.bounds.width,
                          height: node.bounds.height,
                        ),
                      ),
                    ),
                  IgnorePointer(
                    child: SelectionHighlightOverlay(
                      highlights: transformedHighlights,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

List<Widget> buildRenderLayers({
  required PageRenderModel page,
  required ViewerPlatform platform,
  required double scaleX,
  required double scaleY,
}) {
  return [
    for (var index = 0; index < page.nodes.length; index++)
      _buildRenderLayer(
        key: ValueKey('node-$index'),
        node: page.nodes[index],
        platform: platform,
        scaleX: scaleX,
        scaleY: scaleY,
      ),
  ];
}

class _SheetNodePainter extends CustomPainter {
  const _SheetNodePainter({
    required this.page,
    required this.platform,
    required this.visibleRect,
  });

  final PageRenderModel page;
  final ViewerPlatform platform;
  final Rect visibleRect;

  @override
  void paint(Canvas canvas, Size size) {
    final defaultCellFill = Paint()
      ..style = PaintingStyle.fill
      ..color = Colors.white;
    final defaultCellStroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = const Color(0xFFD0D7DE);

    for (final cell in page.sheetCells) {
      final rect = Rect.fromLTWH(
        cell.bounds.x,
        cell.bounds.y,
        cell.bounds.width,
        cell.bounds.height,
      );
      if (!rect.overlaps(visibleRect)) {
        continue;
      }
      canvas.drawRect(rect, defaultCellFill);
      canvas.drawRect(rect, defaultCellStroke);
    }

    for (final node in page.nodes) {
      switch (node) {
        case TextRenderNodeModel():
          final rect = Rect.fromLTWH(
            node.bounds.x,
            node.bounds.y,
            node.bounds.width,
            node.bounds.height,
          );
          if (!rect.overlaps(visibleRect)) {
            continue;
          }
          final painter = buildRenderTextPainter(
            node: node,
            platform: platform,
            scaleX: 1,
            scaleY: 1,
          )..layout();
          painter.paint(canvas, Offset(node.bounds.x, node.bounds.y));
        case BoxRenderNodeModel():
          final rect = Rect.fromLTWH(
            node.bounds.x,
            node.bounds.y,
            node.bounds.width,
            node.bounds.height,
          );
          if (!rect.overlaps(visibleRect)) {
            continue;
          }
          final radius = Radius.circular(node.cornerRadius ?? 0);
          final fillPaint = Paint()..style = PaintingStyle.fill;
          final fillColor = parseColor(node.fillColorHex);
          final gradientEndColor = parseColor(node.gradientEndColorHex);
          if (fillColor != null && gradientEndColor != null) {
            fillPaint.shader = _buildLinearGradientShader(
              rect,
              fillColor,
              gradientEndColor,
              node.gradientAngleDegrees ?? 0,
            );
          } else {
            fillPaint.color = fillColor ?? Colors.transparent;
          }

          final rrect = RRect.fromRectAndRadius(rect, radius);
          if (node.cornerRadius != null && node.cornerRadius! > 0) {
            canvas.drawRRect(rrect, fillPaint);
          } else {
            canvas.drawRect(rect, fillPaint);
          }

          if (node.strokeWidth > 0) {
            final strokePaint = Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = node.strokeWidth
              ..color = parseColor(node.strokeColorHex) ?? Colors.black;
            if (node.cornerRadius != null && node.cornerRadius! > 0) {
              canvas.drawRRect(rrect, strokePaint);
            } else {
              canvas.drawRect(rect, strokePaint);
            }
          }
        case ImageRenderNodeModel():
          continue;
      }
    }
  }

  @override
  bool shouldRepaint(covariant _SheetNodePainter oldDelegate) {
    return oldDelegate.page != page ||
        oldDelegate.platform != platform ||
        oldDelegate.visibleRect != visibleRect;
  }
}

Offset _toPageOffset(Offset localPosition, double scaleX, double scaleY) {
  return Offset(localPosition.dx / scaleX, localPosition.dy / scaleY);
}

Widget _buildRenderLayer({
  required Key key,
  required RenderNodeModel node,
  required ViewerPlatform platform,
  required double scaleX,
  required double scaleY,
}) {
  switch (node) {
    case TextRenderNodeModel():
      return Positioned(
        key: key,
        left: node.bounds.x * scaleX,
        top: node.bounds.y * scaleY,
        width: node.bounds.width * scaleX,
        height: node.bounds.height * scaleY,
        child: IgnorePointer(
          child: CustomPaint(
            painter: _TextNodePainter(
              node: node,
              platform: platform,
              scaleX: scaleX,
              scaleY: scaleY,
            ),
          ),
        ),
      );
    case BoxRenderNodeModel():
      return Positioned(
        key: key,
        left: node.bounds.x * scaleX,
        top: node.bounds.y * scaleY,
        width: node.bounds.width * scaleX,
        height: node.bounds.height * scaleY,
        child: IgnorePointer(
          child: CustomPaint(
            painter: _BoxNodePainter(
              node: node,
              scaleX: scaleX,
              scaleY: scaleY,
            ),
          ),
        ),
      );
    case ImageRenderNodeModel():
      return Positioned(
        key: key,
        left: node.bounds.x * scaleX,
        top: node.bounds.y * scaleY,
        width: node.bounds.width * scaleX,
        height: node.bounds.height * scaleY,
        child: IgnorePointer(
          child: _ImageNodeLayer(
            node: node,
            width: node.bounds.width * scaleX,
            height: node.bounds.height * scaleY,
          ),
        ),
      );
  }
}

class _TextNodePainter extends CustomPainter {
  const _TextNodePainter({
    required this.node,
    required this.platform,
    required this.scaleX,
    required this.scaleY,
  });

  final TextRenderNodeModel node;
  final ViewerPlatform platform;
  final double scaleX;
  final double scaleY;

  @override
  void paint(Canvas canvas, Size size) {
    final painter = buildRenderTextPainter(
      node: node,
      platform: platform,
      scaleX: scaleX,
      scaleY: scaleY,
    )..layout();
    painter.paint(canvas, Offset.zero);
  }

  @override
  bool shouldRepaint(covariant _TextNodePainter oldDelegate) {
    return oldDelegate.node != node ||
        oldDelegate.platform != platform ||
        oldDelegate.scaleX != scaleX ||
        oldDelegate.scaleY != scaleY;
  }
}

class _BoxNodePainter extends CustomPainter {
  const _BoxNodePainter({
    required this.node,
    required this.scaleX,
    required this.scaleY,
  });

  final BoxRenderNodeModel node;
  final double scaleX;
  final double scaleY;

  @override
  void paint(Canvas canvas, Size size) {
    final rect = Offset.zero & size;
    final radius = Radius.circular(
      (node.cornerRadius ?? 0) * math.min(scaleX, scaleY),
    );
    final fillPaint = Paint()..style = PaintingStyle.fill;
    final fillColor = parseColor(node.fillColorHex);
    final gradientEndColor = parseColor(node.gradientEndColorHex);
    if (fillColor != null && gradientEndColor != null) {
      fillPaint.shader = _buildLinearGradientShader(
        rect,
        fillColor,
        gradientEndColor,
        node.gradientAngleDegrees ?? 0,
      );
    } else {
      fillPaint.color = fillColor ?? Colors.transparent;
    }

    final rrect = RRect.fromRectAndRadius(rect, radius);
    if (node.cornerRadius != null && node.cornerRadius! > 0) {
      canvas.drawRRect(rrect, fillPaint);
    } else {
      canvas.drawRect(rect, fillPaint);
    }

    if (node.strokeWidth > 0) {
      final strokePaint = Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = node.strokeWidth * math.min(scaleX, scaleY)
        ..color = parseColor(node.strokeColorHex) ?? Colors.black;
      if (node.cornerRadius != null && node.cornerRadius! > 0) {
        canvas.drawRRect(rrect, strokePaint);
      } else {
        canvas.drawRect(rect, strokePaint);
      }
    }
  }

  @override
  bool shouldRepaint(covariant _BoxNodePainter oldDelegate) {
    return oldDelegate.node != node ||
        oldDelegate.scaleX != scaleX ||
        oldDelegate.scaleY != scaleY;
  }
}

class _ImageNodeLayer extends StatelessWidget {
  const _ImageNodeLayer({
    required this.node,
    required this.width,
    required this.height,
  });

  final ImageRenderNodeModel node;
  final double width;
  final double height;

  @override
  Widget build(BuildContext context) {
    if (node.dataBase64 == null) {
      return ColoredBox(
        color: const Color(0xFFE9EEF7),
        child: Center(
          child: Text(
            node.description ?? node.resourceId,
            textAlign: TextAlign.center,
            style: const TextStyle(color: Color(0xFF42526B), fontSize: 10),
          ),
        ),
      );
    }

    final crop = node.crop;
    final visibleWidthFactor = crop == null
        ? 1.0
        : (1.0 - crop.left - crop.right).clamp(0.01, 1.0);
    final visibleHeightFactor = crop == null
        ? 1.0
        : (1.0 - crop.top - crop.bottom).clamp(0.01, 1.0);
    final childWidth = width / visibleWidthFactor;
    final childHeight = height / visibleHeightFactor;
    final offsetX = crop == null ? 0.0 : -(crop.left * childWidth);
    final offsetY = crop == null ? 0.0 : -(crop.top * childHeight);

    Widget image = Image.memory(
      base64Decode(node.dataBase64!),
      width: childWidth,
      height: childHeight,
      fit: BoxFit.fill,
      filterQuality: FilterQuality.high,
      gaplessPlayback: true,
      errorBuilder: (context, error, stackTrace) => ColoredBox(
        color: const Color(0xFFE9EEF7),
        child: Center(
          child: Text(
            node.description ?? node.resourceId,
            textAlign: TextAlign.center,
            style: const TextStyle(color: Color(0xFF42526B), fontSize: 10),
          ),
        ),
      ),
    );

    if (node.flipHorizontal || node.flipVertical) {
      image = Transform(
        alignment: Alignment.center,
        transform: Matrix4.diagonal3Values(
          node.flipHorizontal ? -1.0 : 1.0,
          node.flipVertical ? -1.0 : 1.0,
          1.0,
        ),
        child: image,
      );
    }

    return ClipRect(
      child: Stack(
        children: [
          Positioned(
            left: offsetX,
            top: offsetY,
            width: childWidth,
            height: childHeight,
            child: image,
          ),
        ],
      ),
    );
  }
}

Shader _buildLinearGradientShader(
  Rect rect,
  Color startColor,
  Color endColor,
  double angleDegrees,
) {
  final radians = angleDegrees * 3.141592653589793 / 180.0;
  final dx = math.cos(radians);
  final dy = math.sin(radians);
  final center = rect.center;
  final extent = Offset(rect.width * dx, rect.height * dy) / 2;
  final start = center - extent;
  final end = center + extent;
  return ui.Gradient.linear(start, end, [startColor, endColor]);
}

@visibleForTesting
TextPainter buildRenderTextPainter({
  required TextRenderNodeModel node,
  required ViewerPlatform platform,
  required double scaleX,
  required double scaleY,
}) {
  final script = scriptKindForText(node.text);
  final fontFamily = resolveRenderableFontFamily(
    platform: platform,
    requested: node.style.fontFamily,
    script: script,
  );

  final baseColor = parseColor(node.style.colorHex) ?? Colors.black;
  final gradientEndColor = parseColor(node.style.gradientEndColorHex);
  final Paint? foreground;
  if (gradientEndColor == null) {
    foreground = null;
  } else {
    foreground = Paint()
      ..shader = _buildLinearGradientShader(
        Rect.fromLTWH(
          0,
          0,
          node.bounds.width * scaleX,
          node.bounds.height * scaleY,
        ),
        baseColor,
        gradientEndColor,
        node.style.gradientAngleDegrees ?? 0,
      );
  }

  return TextPainter(
    text: TextSpan(
      text: node.text,
      style: TextStyle(
        fontFamily: fontFamily,
        fontFamilyFallback: resolveFontFamilyFallbacks(
          platform: platform,
          requested: node.style.fontFamily,
          script: script,
        ),
        fontSize: node.style.fontSize * scaleY,
        fontWeight: node.style.bold ? FontWeight.w700 : FontWeight.w400,
        fontStyle: node.style.italic ? FontStyle.italic : FontStyle.normal,
        color: foreground == null ? baseColor : null,
        foreground: foreground,
        decoration: node.style.underline ? TextDecoration.underline : null,
        decorationColor: baseColor,
      ),
    ),
    textDirection: TextDirection.ltr,
    maxLines: 1,
  );
}

Color? parseColor(String? hex) {
  if (hex == null) {
    return null;
  }
  final normalized = hex.replaceFirst('#', '');
  if (normalized.length == 6) {
    return Color(int.parse('FF$normalized', radix: 16));
  }
  if (normalized.length == 8) {
    final rgb = normalized.substring(0, 6);
    final alpha = normalized.substring(6, 8);
    return Color(int.parse('$alpha$rgb', radix: 16));
  }
  return null;
}

ViewerPlatform viewerPlatformForTargetPlatform(TargetPlatform platform) {
  switch (platform) {
    case TargetPlatform.iOS:
      return ViewerPlatform.ios;
    case TargetPlatform.android:
      return ViewerPlatform.android;
    case TargetPlatform.macOS:
      return ViewerPlatform.macOs;
    case TargetPlatform.windows:
      return ViewerPlatform.windows;
    case TargetPlatform.linux:
      return ViewerPlatform.android;
    case TargetPlatform.fuchsia:
      return ViewerPlatform.android;
  }
}

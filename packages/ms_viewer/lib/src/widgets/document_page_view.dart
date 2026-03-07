import 'dart:convert';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart'
    show TargetPlatform, defaultTargetPlatform, visibleForTesting;
import 'package:flutter/material.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import '../painting/font_fallback_policy.dart';
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
                    painter: _PageRenderPainter(
                      page: page,
                      platform: _viewerPlatformForTargetPlatform(
                        defaultTargetPlatform,
                      ),
                    ),
                  ),
                  ..._buildImageLayers(
                    page: page,
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
          );
        },
      ),
    );
  }
}

List<Widget> _buildImageLayers({
  required PageRenderModel page,
  required double scaleX,
  required double scaleY,
}) {
  final layers = <Widget>[];

  for (final node in page.nodes) {
    if (node is! ImageRenderNodeModel || node.dataBase64 == null) {
      continue;
    }

    layers.add(
      Positioned(
        left: node.bounds.x * scaleX,
        top: node.bounds.y * scaleY,
        width: node.bounds.width * scaleX,
        height: node.bounds.height * scaleY,
        child: Image.memory(
          base64Decode(node.dataBase64!),
          fit: BoxFit.contain,
          filterQuality: FilterQuality.medium,
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
        ),
      ),
    );
  }

  return layers;
}

Offset _toPageOffset(Offset localPosition, double scaleX, double scaleY) {
  return Offset(localPosition.dx / scaleX, localPosition.dy / scaleY);
}

class _PageRenderPainter extends CustomPainter {
  _PageRenderPainter({required this.page, required this.platform});

  final PageRenderModel page;
  final ViewerPlatform platform;

  @override
  void paint(Canvas canvas, Size size) {
    final scaleX = size.width / page.width;
    final scaleY = size.height / page.height;

    for (final node in page.nodes) {
      switch (node) {
        case TextRenderNodeModel():
          final painter = buildRenderTextPainter(
            node: node,
            platform: platform,
            scaleX: scaleX,
            scaleY: scaleY,
          )..layout();
          painter.paint(
            canvas,
            Offset(node.bounds.x * scaleX, node.bounds.y * scaleY),
          );
        case BoxRenderNodeModel():
          final rect = Rect.fromLTWH(
            node.bounds.x * scaleX,
            node.bounds.y * scaleY,
            node.bounds.width * scaleX,
            node.bounds.height * scaleY,
          );
          final paint = Paint()..style = PaintingStyle.fill;
          final fillColor = _parseColor(node.fillColorHex);
          final gradientEndColor = _parseColor(node.gradientEndColorHex);
          if (fillColor != null && gradientEndColor != null) {
            paint.shader = _buildLinearGradientShader(
              rect,
              fillColor,
              gradientEndColor,
              node.gradientAngleDegrees ?? 0,
            );
          } else {
            paint.color = fillColor ?? Colors.transparent;
          }
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
          if (node.dataBase64 != null) {
            continue;
          }
          final rect = Rect.fromLTWH(
            node.bounds.x * scaleX,
            node.bounds.y * scaleY,
            node.bounds.width * scaleX,
            node.bounds.height * scaleY,
          );
          canvas.drawRect(rect, Paint()..color = const Color(0xFFE9EEF7));
          final painter = TextPainter(
            text: TextSpan(
              text: node.description ?? node.resourceId,
              style: const TextStyle(color: Color(0xFF42526B), fontSize: 10),
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

  final baseColor = _parseColor(node.style.colorHex) ?? Colors.black;
  final gradientEndColor = _parseColor(node.style.gradientEndColorHex);
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
      ),
    ),
    textDirection: TextDirection.ltr,
    maxLines: 1,
  );
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

ViewerPlatform _viewerPlatformForTargetPlatform(TargetPlatform platform) {
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

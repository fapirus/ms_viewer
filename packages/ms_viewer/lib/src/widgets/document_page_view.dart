import 'package:flutter/material.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import 'render_node_canvas.dart';

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

          return DecoratedBox(
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
            child: RenderNodeCanvas(
              page: page,
              canvasWidth: size.width,
              canvasHeight: size.height,
              scaleX: scaleX,
              scaleY: scaleY,
              highlights: highlights,
              backgroundColor: Colors.white,
              clipBehavior: Clip.hardEdge,
              onSelectionStart: onSelectionStart,
              onSelectionUpdate: onSelectionUpdate,
              onSelectionEnd: onSelectionEnd,
            ),
          );
        },
      ),
    );
  }
}

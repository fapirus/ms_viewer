import 'package:flutter/material.dart';

class SelectionHighlightOverlay extends StatelessWidget {
  const SelectionHighlightOverlay({
    super.key,
    required this.highlights,
    this.color = const Color(0x663B82F6),
  });

  final List<Rect> highlights;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: highlights
          .map(
            (rect) => Positioned(
              left: rect.left,
              top: rect.top,
              width: rect.width,
              height: rect.height,
              child: DecoratedBox(
                decoration: BoxDecoration(color: color),
              ),
            ),
          )
          .toList(),
    );
  }
}

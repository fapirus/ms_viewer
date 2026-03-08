class RectModel {
  const RectModel({
    required this.x,
    required this.y,
    required this.width,
    required this.height,
  });

  final double x;
  final double y;
  final double width;
  final double height;

  factory RectModel.fromJson(Map<String, Object?> json) {
    return RectModel(
      x: (json['x'] as num).toDouble(),
      y: (json['y'] as num).toDouble(),
      width: (json['width'] as num).toDouble(),
      height: (json['height'] as num).toDouble(),
    );
  }
}

class TextRangeModel {
  const TextRangeModel({required this.start, required this.end});

  final int start;
  final int end;

  factory TextRangeModel.fromJson(Map<String, Object?> json) {
    return TextRangeModel(start: json['start'] as int, end: json['end'] as int);
  }
}

class TextStyleModel {
  const TextStyleModel({
    required this.fontFamily,
    required this.fontSize,
    required this.bold,
    required this.italic,
    required this.underline,
    required this.colorHex,
    required this.gradientEndColorHex,
    required this.gradientAngleDegrees,
  });

  final String fontFamily;
  final double fontSize;
  final bool bold;
  final bool italic;
  final bool underline;
  final String colorHex;
  final String? gradientEndColorHex;
  final double? gradientAngleDegrees;

  factory TextStyleModel.fromJson(Map<String, Object?> json) {
    return TextStyleModel(
      fontFamily: json['fontFamily'] as String,
      fontSize: (json['fontSize'] as num).toDouble(),
      bold: json['bold'] as bool? ?? false,
      italic: json['italic'] as bool? ?? false,
      underline: json['underline'] as bool? ?? false,
      colorHex: json['colorHex'] as String,
      gradientEndColorHex: json['gradientEndColorHex'] as String?,
      gradientAngleDegrees: (json['gradientAngleDegrees'] as num?)?.toDouble(),
    );
  }
}

class SelectionAnchorModel {
  const SelectionAnchorModel({
    required this.nodeIndex,
    required this.charIndex,
    required this.x,
    required this.y,
  });

  final int nodeIndex;
  final int charIndex;
  final double x;
  final double y;

  factory SelectionAnchorModel.fromJson(Map<String, Object?> json) {
    return SelectionAnchorModel(
      nodeIndex: json['nodeIndex'] as int,
      charIndex: json['charIndex'] as int,
      x: (json['x'] as num).toDouble(),
      y: (json['y'] as num).toDouble(),
    );
  }
}

sealed class RenderNodeModel {
  const RenderNodeModel();

  factory RenderNodeModel.fromJson(Map<String, Object?> json) {
    switch (json['type']) {
      case 'text':
        return TextRenderNodeModel.fromJson(json);
      case 'image':
        return ImageRenderNodeModel.fromJson(json);
      case 'box':
        return BoxRenderNodeModel.fromJson(json);
    }

    throw ArgumentError('Unknown render node type: ${json['type']}');
  }
}

class TextRenderNodeModel extends RenderNodeModel {
  const TextRenderNodeModel({
    required this.text,
    required this.bounds,
    required this.style,
    required this.range,
  });

  final String text;
  final RectModel bounds;
  final TextStyleModel style;
  final TextRangeModel range;

  factory TextRenderNodeModel.fromJson(Map<String, Object?> json) {
    return TextRenderNodeModel(
      text: json['text'] as String,
      bounds: RectModel.fromJson(json['bounds'] as Map<String, Object?>),
      style: TextStyleModel.fromJson(json['style'] as Map<String, Object?>),
      range: TextRangeModel.fromJson(json['range'] as Map<String, Object?>),
    );
  }
}

class ImageRenderNodeModel extends RenderNodeModel {
  const ImageRenderNodeModel({
    required this.resourceId,
    required this.description,
    required this.contentType,
    required this.dataBase64,
    required this.bounds,
    required this.crop,
    required this.flipHorizontal,
    required this.flipVertical,
  });

  final String resourceId;
  final String? description;
  final String? contentType;
  final String? dataBase64;
  final RectModel bounds;
  final ImageCropInsetsModel? crop;
  final bool flipHorizontal;
  final bool flipVertical;

  factory ImageRenderNodeModel.fromJson(Map<String, Object?> json) {
    final cropJson = json['crop'] as Map<String, Object?>?;
    return ImageRenderNodeModel(
      resourceId: json['resourceId'] as String,
      description: json['description'] as String?,
      contentType: json['contentType'] as String?,
      dataBase64: json['dataBase64'] as String?,
      bounds: RectModel.fromJson(json['bounds'] as Map<String, Object?>),
      crop: cropJson == null ? null : ImageCropInsetsModel.fromJson(cropJson),
      flipHorizontal: json['flipHorizontal'] as bool? ?? false,
      flipVertical: json['flipVertical'] as bool? ?? false,
    );
  }
}

class ImageCropInsetsModel {
  const ImageCropInsetsModel({
    required this.left,
    required this.top,
    required this.right,
    required this.bottom,
  });

  final double left;
  final double top;
  final double right;
  final double bottom;

  factory ImageCropInsetsModel.fromJson(Map<String, Object?> json) {
    return ImageCropInsetsModel(
      left: (json['left'] as num).toDouble(),
      top: (json['top'] as num).toDouble(),
      right: (json['right'] as num).toDouble(),
      bottom: (json['bottom'] as num).toDouble(),
    );
  }
}

class BoxRenderNodeModel extends RenderNodeModel {
  const BoxRenderNodeModel({
    required this.bounds,
    required this.fillColorHex,
    required this.gradientEndColorHex,
    required this.gradientAngleDegrees,
    required this.strokeColorHex,
    required this.strokeWidth,
    required this.cornerRadius,
  });

  final RectModel bounds;
  final String? fillColorHex;
  final String? gradientEndColorHex;
  final double? gradientAngleDegrees;
  final String? strokeColorHex;
  final double strokeWidth;
  final double? cornerRadius;

  factory BoxRenderNodeModel.fromJson(Map<String, Object?> json) {
    return BoxRenderNodeModel(
      bounds: RectModel.fromJson(json['bounds'] as Map<String, Object?>),
      fillColorHex: json['fillColorHex'] as String?,
      gradientEndColorHex: json['gradientEndColorHex'] as String?,
      gradientAngleDegrees: (json['gradientAngleDegrees'] as num?)?.toDouble(),
      strokeColorHex: json['strokeColorHex'] as String?,
      strokeWidth: (json['strokeWidth'] as num).toDouble(),
      cornerRadius: (json['cornerRadius'] as num?)?.toDouble(),
    );
  }
}

class PageRenderModel {
  const PageRenderModel({
    required this.pageIndex,
    required this.width,
    required this.height,
    required this.nodes,
    required this.selectionAnchors,
    required this.sheetViewport,
  });

  final int pageIndex;
  final double width;
  final double height;
  final List<RenderNodeModel> nodes;
  final List<SelectionAnchorModel> selectionAnchors;
  final SheetViewportModel? sheetViewport;

  factory PageRenderModel.fromJson(Map<String, Object?> json) {
    final nodesJson = json['nodes'] as List<Object?>? ?? const [];
    final anchorsJson = json['selectionAnchors'] as List<Object?>? ?? const [];
    final sheetViewportJson = json['sheetViewport'] as Map<String, Object?>?;

    return PageRenderModel(
      pageIndex: json['pageIndex'] as int,
      width: (json['width'] as num).toDouble(),
      height: (json['height'] as num).toDouble(),
      nodes: nodesJson
          .cast<Map<String, Object?>>()
          .map(RenderNodeModel.fromJson)
          .toList(),
      selectionAnchors: anchorsJson
          .cast<Map<String, Object?>>()
          .map(SelectionAnchorModel.fromJson)
          .toList(),
      sheetViewport: sheetViewportJson == null
          ? null
          : SheetViewportModel.fromJson(sheetViewportJson),
    );
  }
}

class SheetViewportModel {
  const SheetViewportModel({
    required this.window,
    required this.effectiveBounds,
    required this.frozenPane,
    required this.visibleRows,
    required this.visibleColumns,
  });

  final SheetBoundsModel window;
  final SheetBoundsModel effectiveBounds;
  final SheetFrozenPaneModel? frozenPane;
  final List<int> visibleRows;
  final List<int> visibleColumns;

  factory SheetViewportModel.fromJson(Map<String, Object?> json) {
    final frozenPaneJson = json['frozenPane'] as Map<String, Object?>?;
    return SheetViewportModel(
      window: SheetBoundsModel.fromJson(json['window'] as Map<String, Object?>),
      effectiveBounds: SheetBoundsModel.fromJson(
        json['effectiveBounds'] as Map<String, Object?>,
      ),
      frozenPane: frozenPaneJson == null
          ? null
          : SheetFrozenPaneModel.fromJson(frozenPaneJson),
      visibleRows: (json['visibleRows'] as List<Object?>? ?? const [])
          .cast<int>(),
      visibleColumns: (json['visibleColumns'] as List<Object?>? ?? const [])
          .cast<int>(),
    );
  }
}

class SheetBoundsModel {
  const SheetBoundsModel({
    required this.startRow,
    required this.endRow,
    required this.startColumn,
    required this.endColumn,
  });

  final int startRow;
  final int endRow;
  final int startColumn;
  final int endColumn;

  factory SheetBoundsModel.fromJson(Map<String, Object?> json) {
    return SheetBoundsModel(
      startRow: json['startRow'] as int,
      endRow: json['endRow'] as int,
      startColumn: json['startColumn'] as int,
      endColumn: json['endColumn'] as int,
    );
  }
}

class SheetFrozenPaneModel {
  const SheetFrozenPaneModel({
    required this.frozenRows,
    required this.frozenColumns,
    required this.topLeftCell,
  });

  final int frozenRows;
  final int frozenColumns;
  final String? topLeftCell;

  factory SheetFrozenPaneModel.fromJson(Map<String, Object?> json) {
    return SheetFrozenPaneModel(
      frozenRows: json['frozenRows'] as int,
      frozenColumns: json['frozenColumns'] as int,
      topLeftCell: json['topLeftCell'] as String?,
    );
  }
}

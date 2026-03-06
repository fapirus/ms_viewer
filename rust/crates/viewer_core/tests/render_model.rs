use viewer_core::model::{
    PageRenderModel, Rect, RenderNode, SelectionAnchor, TextNode, TextRange, TextStyle,
};

#[test]
fn page_render_model_serializes_with_expected_shape() {
    let model = PageRenderModel {
        page_index: 0,
        width: 595.0,
        height: 842.0,
        nodes: vec![RenderNode::Text(TextNode {
            text: "Hello".to_string(),
            bounds: Rect {
                x: 24.0,
                y: 32.0,
                width: 120.0,
                height: 18.0,
            },
            style: TextStyle {
                font_family: "Calibri".to_string(),
                font_size: 11.0,
                bold: false,
                italic: false,
                color_hex: "#000000".to_string(),
            },
            range: TextRange { start: 0, end: 5 },
        })],
        selection_anchors: vec![SelectionAnchor {
            node_index: 0,
            char_index: 0,
            x: 24.0,
            y: 32.0,
        }],
    };

    let value = serde_json::to_value(&model).expect("model should serialize");

    assert_eq!(value["pageIndex"], 0);
    assert_eq!(value["nodes"][0]["type"], "text");
    assert_eq!(value["nodes"][0]["range"]["start"], 0);
    assert_eq!(value["selectionAnchors"][0]["charIndex"], 0);
}

#[test]
fn page_render_model_round_trips() {
    let json = r##"{
      "pageIndex": 1,
      "width": 1024.0,
      "height": 768.0,
      "nodes": [
        {
          "type": "box",
          "bounds": {"x": 0.0, "y": 0.0, "width": 1024.0, "height": 768.0},
          "fillColorHex": "#FFFFFF",
          "strokeColorHex": null,
          "strokeWidth": 0.0
        }
      ],
      "selectionAnchors": []
    }"##;

    let model: PageRenderModel = serde_json::from_str(json).expect("json should deserialize");

    assert_eq!(model.page_index, 1);
    assert_eq!(model.nodes.len(), 1);
}

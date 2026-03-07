use viewer_core::model::{
    Block, FloatingTablePosition, ImageReference, ListKind, ListMarker, ParagraphAlignment,
    ParagraphMetrics, TableAlignment, TableAnchor, TableCell, TableCellMerge,
    TableHorizontalPosition, TableLayout, TableRow, TableVerticalPosition, TextRun, TextStyle,
};

#[test]
fn shared_text_model_can_be_constructed() {
    let style = TextStyle {
        font_family: "Calibri".to_string(),
        font_size: 11.0,
        bold: false,
        italic: false,
        color_hex: "#000000".to_string(),
        gradient_end_color_hex: None,
        gradient_angle_degrees: None,
    };

    let paragraph = Block::Paragraph {
        runs: vec![TextRun {
            text: "Hello world".to_string(),
            style: style.clone(),
        }],
        list: Some(ListMarker {
            level: 0,
            kind: ListKind::Bullet,
            num_id: 1,
        }),
        metrics: ParagraphMetrics {
            alignment: ParagraphAlignment::Left,
            line_height: Some(14.0),
            spacing_before: 6.0,
            spacing_after: 8.0,
        },
    };

    let image = Block::Image {
        image: ImageReference {
            resource_id: "rId5".to_string(),
            description: Some("logo".to_string()),
            content_type: Some("image/png".to_string()),
            display_width: Some(120.0),
            display_height: Some(64.0),
        },
        alignment: ParagraphAlignment::Center,
    };
    let table = Block::Table {
        rows: vec![TableRow {
            cells: vec![TableCell {
                blocks: vec![Block::Paragraph {
                    runs: vec![TextRun {
                        text: "Cell".to_string(),
                        style: style.clone(),
                    }],
                    list: None,
                    metrics: ParagraphMetrics {
                        alignment: ParagraphAlignment::Left,
                        line_height: Some(14.0),
                        spacing_before: 0.0,
                        spacing_after: 0.0,
                    },
                }],
                column_span: 2,
                row_merge: Some(TableCellMerge::Restart),
            }],
        }],
        column_widths: vec![120.0, 240.0],
        layout: TableLayout {
            preferred_width: Some(240.0),
            alignment: TableAlignment::Center,
            floating: Some(FloatingTablePosition {
                horz_anchor: TableAnchor::Margin,
                vert_anchor: TableAnchor::Page,
                x: None,
                y: Some(480.0),
                x_position: Some(TableHorizontalPosition::Center),
                y_position: Some(TableVerticalPosition::Top),
                left_from_text: 6.0,
                right_from_text: 6.0,
            }),
        },
    };

    match paragraph {
        Block::Paragraph {
            runs,
            list,
            metrics,
        } => {
            assert_eq!(runs.len(), 1);
            assert_eq!(runs[0].style, style);
            assert_eq!(list.unwrap().kind, ListKind::Bullet);
            assert_eq!(metrics.spacing_before, 6.0);
        }
        _ => panic!("expected paragraph block"),
    }

    match image {
        Block::Image { image, alignment } => {
            assert_eq!(image.resource_id, "rId5");
            assert_eq!(alignment, ParagraphAlignment::Center);
        }
        _ => panic!("expected image block"),
    }

    match table {
        Block::Table {
            rows,
            column_widths,
            layout,
        } => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].cells[0].column_span, 2);
            assert_eq!(column_widths, vec![120.0, 240.0]);
            assert_eq!(layout.preferred_width, Some(240.0));
            assert_eq!(layout.alignment, TableAlignment::Center);
        }
        _ => panic!("expected table block"),
    }
}

#[test]
fn shared_text_model_round_trips_via_serde() {
    let block = Block::Paragraph {
        runs: vec![TextRun {
            text: "Example".to_string(),
            style: TextStyle {
                font_family: "Calibri".to_string(),
                font_size: 11.0,
                bold: true,
                italic: false,
                color_hex: "#222222".to_string(),
                gradient_end_color_hex: None,
                gradient_angle_degrees: None,
            },
        }],
        list: Some(ListMarker {
            level: 1,
            kind: ListKind::Decimal,
            num_id: 9,
        }),
        metrics: ParagraphMetrics {
            alignment: ParagraphAlignment::Center,
            line_height: Some(16.0),
            spacing_before: 4.0,
            spacing_after: 10.0,
        },
    };

    let json = serde_json::to_string(&block).expect("block should serialize");
    let decoded: Block = serde_json::from_str(&json).expect("block should deserialize");

    assert_eq!(decoded, block);
}

use viewer_core::model::{
    Block, ImageReference, ListKind, ListMarker, TableCell, TableCellMerge, TableRow, TextRun,
    TextStyle,
};

#[test]
fn shared_text_model_can_be_constructed() {
    let style = TextStyle {
        font_family: "Calibri".to_string(),
        font_size: 11.0,
        bold: false,
        italic: false,
        color_hex: "#000000".to_string(),
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
    };

    let image = Block::Image {
        image: ImageReference {
            resource_id: "rId5".to_string(),
            description: Some("logo".to_string()),
            content_type: Some("image/png".to_string()),
            display_width: Some(120.0),
            display_height: Some(64.0),
        },
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
                }],
                column_span: 2,
                row_merge: Some(TableCellMerge::Restart),
            }],
        }],
        column_widths: vec![120.0, 240.0],
    };

    match paragraph {
        Block::Paragraph { runs, list } => {
            assert_eq!(runs.len(), 1);
            assert_eq!(runs[0].style, style);
            assert_eq!(list.unwrap().kind, ListKind::Bullet);
        }
        _ => panic!("expected paragraph block"),
    }

    match image {
        Block::Image { image } => {
            assert_eq!(image.resource_id, "rId5");
        }
        _ => panic!("expected image block"),
    }

    match table {
        Block::Table {
            rows,
            column_widths,
        } => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].cells[0].column_span, 2);
            assert_eq!(column_widths, vec![120.0, 240.0]);
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
            },
        }],
        list: Some(ListMarker {
            level: 1,
            kind: ListKind::Decimal,
            num_id: 9,
        }),
    };

    let json = serde_json::to_string(&block).expect("block should serialize");
    let decoded: Block = serde_json::from_str(&json).expect("block should deserialize");

    assert_eq!(decoded, block);
}

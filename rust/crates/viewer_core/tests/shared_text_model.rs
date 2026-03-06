use viewer_core::model::{Block, ImageReference, ListKind, ListMarker, TextRun, TextStyle};

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
        },
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

use std::path::PathBuf;

use format_pptx::{
    build_slide_render_model, parse_pptx, parse_slide_basic_shapes, parse_slide_images,
    search_slides,
};
use viewer_core::archive::OoxmlArchive;
use viewer_core::model::RenderNode;

fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(relative)
}

#[test]
fn review_fixture_text_shapes_opens_and_supports_render_search_selection() {
    let archive = OoxmlArchive::open_path(fixture_path("fixtures/pptx/pptx_text_shapes.pptx"))
        .expect("fixture archive");
    let slide_tree = parse_pptx(&archive).expect("slide tree");

    assert_eq!(slide_tree.slides.len(), 2);

    let first_page = build_slide_render_model(&archive, &slide_tree, 0).expect("page 0");
    let text_nodes: Vec<_> = first_page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(node) => Some(node),
            _ => None,
        })
        .collect();
    let box_nodes: Vec<_> = first_page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Box(node) => Some(node),
            _ => None,
        })
        .collect();

    assert!(!text_nodes.is_empty());
    assert!(!box_nodes.is_empty());
    assert!(!first_page.selection_anchors.is_empty());

    let second_page = build_slide_render_model(&archive, &slide_tree, 1).expect("page 1");
    let second_text_nodes: Vec<_> = second_page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Text(node) => Some(node),
            _ => None,
        })
        .collect();
    assert!(second_text_nodes.len() >= 3);
    assert!(second_text_nodes
        .windows(2)
        .all(|pair| pair[0].range.end == pair[1].range.start));

    let shapes =
        parse_slide_basic_shapes(&archive, &slide_tree.slides[0].part_name).expect("shapes");
    assert_eq!(shapes.len(), 1);

    let matches = search_slides(&archive, &slide_tree, "revenue").expect("search");
    assert_eq!(matches.len(), 2);
    assert!(matches.iter().all(|item| item.page_index == 1));
}

#[test]
fn review_fixture_theme_layout_image_links_open_and_render() {
    let archive =
        OoxmlArchive::open_path(fixture_path("fixtures/pptx/pptx_theme_layout_images.pptx"))
            .expect("fixture archive");
    let slide_tree = parse_pptx(&archive).expect("slide tree");

    assert_eq!(slide_tree.slides.len(), 1);
    assert_eq!(slide_tree.slide_masters.len(), 1);
    assert_eq!(
        slide_tree.slides[0].layout_part_name.as_deref(),
        Some("ppt/slideLayouts/slideLayout1.xml")
    );
    assert_eq!(
        slide_tree.slide_masters[0].theme_part_name.as_deref(),
        Some("ppt/theme/theme1.xml")
    );

    let images = parse_slide_images(&archive, &slide_tree.slides[0].part_name).expect("images");
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].image.content_type.as_deref(), Some("image/png"));

    let page = build_slide_render_model(&archive, &slide_tree, 0).expect("page");
    let image_nodes: Vec<_> = page
        .nodes
        .iter()
        .filter_map(|node| match node {
            RenderNode::Image(node) => Some(node),
            _ => None,
        })
        .collect();
    assert_eq!(image_nodes.len(), 1);
    assert!(image_nodes[0].data_base64.is_some());
    assert_eq!(image_nodes[0].description.as_deref(), Some("Fixture image"));
}

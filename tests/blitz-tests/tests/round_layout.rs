//! `DocumentConfig::round_layout`: `final_layout` is rounded to whole CSS pixels by default, and
//! keeps the exact fractional layout when rounding is off (PDF and print output).

use blitz_dom::DocumentConfig;
use blitz_html::{HtmlDocument, HtmlProvider};
use blitz_traits::shell::{ColorScheme, Viewport};
use std::sync::Arc;

const HTML: &str = r#"<html><body style="margin:0">
    <div style="padding-left:10.25px; width:300px">
        <div id="box" style="width:100.5px; height:20.4px;"></div>
        <div id="abs-parent" style="position:relative; height:10px">
            <div id="abs" style="position:absolute; left:3.3px; top:0.6px; width:7.7px; height:1px"></div>
        </div>
    </div>
</body></html>"#;

fn layout_doc(round_layout: Option<bool>) -> HtmlDocument {
    let mut doc = HtmlDocument::from_html(
        HTML,
        DocumentConfig {
            viewport: Some(Viewport::new(800, 600, 1.0, ColorScheme::Light)),
            html_parser_provider: Some(Arc::new(HtmlProvider) as _),
            round_layout,
            ..Default::default()
        },
    );
    doc.resolve(0.0);
    doc
}

fn layout(doc: &HtmlDocument, selector: &str) -> taffy::Layout {
    let id = doc.query_selector(selector).unwrap().expect(selector);
    *doc.get_node(id).unwrap().final_layout()
}

/// With the `writing-mode` feature, positions go through absolute coordinates and back, so allow
/// float error.
fn assert_near(actual: &[f32], expected: &[f32]) {
    let near = actual
        .iter()
        .zip(expected)
        .all(|(a, e)| (a - e).abs() < 1e-4);
    assert!(near, "{actual:?} != {expected:?}");
}

#[test]
fn layout_is_rounded_by_default() {
    let doc = layout_doc(None);
    let b = layout(&doc, "#box");
    assert_eq!(
        (b.location.x, b.size.width, b.size.height),
        (10.0, 101.0, 20.0)
    );
    // Edges are rounded in absolute coordinates: 10.25 + 3.3 = 13.55 → 14, the parent's 10.25 → 10.
    let a = layout(&doc, "#abs");
    assert_eq!((a.location.x, a.location.y, a.size.width), (4.0, 1.0, 7.0));
}

#[test]
fn layout_keeps_fractions_without_rounding() {
    let doc = layout_doc(Some(false));
    let b = layout(&doc, "#box");
    assert_near(
        &[b.location.x, b.size.width, b.size.height],
        &[10.25, 100.5, 20.4],
    );
    let a = layout(&doc, "#abs");
    assert_near(
        &[a.location.x, a.location.y, a.size.width],
        &[3.3, 0.6, 7.7],
    );
}

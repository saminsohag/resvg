// Copyright 2024 the Resvg Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Regression test for complex-script (Indic) glyph advance.
//!
//! Ordering/itemization of mixed Latin+Indic runs is covered by the
//! `text_text_glyph_splitting` reference-image test in the `resvg` crate.

#[cfg(feature = "text")]
fn tree_with_devanagari(text: &str) -> usvg::Tree {
    let mut opt = usvg::Options::default();
    opt.fontdb_mut()
        .load_font_file("../resvg/tests/fonts/NotoSansDevanagari-Regular.ttf")
        .expect("Devanagari test font must be present");

    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 100">
             <text x="0" y="50" font-family="Noto Sans Devanagari" font-size="40">{text}</text>
           </svg>"#
    );
    usvg::Tree::from_str(&svg, &opt).expect("parse")
}

#[cfg(feature = "text")]
fn glyph_origins(tree: &usvg::Tree) -> Vec<f32> {
    fn walk(node: &usvg::Node, out: &mut Vec<f32>) {
        match node {
            usvg::Node::Text(text) => {
                for span in text.layouted() {
                    for g in &span.positioned_glyphs {
                        out.push(g.transform().tx);
                    }
                }
            }
            usvg::Node::Group(group) => {
                for child in group.children() {
                    walk(child, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(&usvg::Node::Group(Box::new(tree.root().clone())), &mut out);
    out
}

// A Devanagari consonant plus a pre-base vowel sign (U+093F) forms a cluster of
// two spacing glyphs: the vowel is reordered before the consonant. That cluster
// must advance by the SUM of the two glyph advances, not the max — using the max
// makes each cluster too narrow so the following cluster is laid out on top of
// it (overlapping text). `किमिति` chains four such clusters, so the shortfall
// accumulates into a clearly measurable total width.
#[cfg(feature = "text")]
#[test]
fn indic_pre_base_vowel_does_not_overlap() {
    let origins = glyph_origins(&tree_with_devanagari("किमिति"));

    assert!(
        origins.len() >= 6,
        "expected the Indic text to produce glyphs, got {}",
        origins.len()
    );

    // Glyph origins must never move backwards: a cluster starting left of where
    // the previous cluster ends is the visible overlap.
    for w in origins.windows(2) {
        assert!(
            w[1] + 0.01 >= w[0],
            "glyph origins moved backwards (overlap): {origins:?}"
        );
    }

    // Measured: the max-advance bug collapses this word to ~64.8px wide, the
    // correct summed advance spreads it to ~85.5px. 75px sits between, so the
    // test fails on the bug and passes on the fix with margin either side.
    let rightmost = origins.iter().cloned().fold(0.0_f32, f32::max);
    assert!(
        rightmost > 75.0,
        "text collapsed — rightmost glyph origin was only {rightmost:.1}px \
         (clusters advancing by max instead of sum)"
    );
}

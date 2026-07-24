// Copyright 2024 the Resvg Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Regression tests for complex-script (Indic) glyph layout.
//!
//! A cluster that contains several *spacing* glyphs — e.g. a Devanagari/Bengali
//! consonant plus a pre-base vowel sign — must advance by the SUM of its glyph
//! advances, not the max. Using the max makes the cluster too narrow so the
//! next cluster is drawn on top of it (overlapping text).

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

// `कि` = क (consonant) + ि (pre-base vowel sign, U+093F). The vowel sign is
// reordered before the consonant and both are spacing glyphs sharing one
// cluster. `किमिति` chains several such clusters so any per-cluster
// under-advance accumulates into a clearly measurable, overlapping run.
#[cfg(feature = "text")]
#[test]
fn indic_pre_base_vowel_does_not_overlap() {
    let tree = tree_with_devanagari("किमिति");
    let origins = glyph_origins(&tree);

    assert!(
        origins.len() >= 6,
        "expected the Indic text to produce glyphs, got {}",
        origins.len()
    );

    // No glyph may start to the left of a glyph that precedes it in the run:
    // that is exactly the overlap the max-advance bug produced.
    for w in origins.windows(2) {
        assert!(
            w[1] + 0.01 >= w[0],
            "glyph origins must not move backwards (overlap): {origins:?}"
        );
    }

    // The whole word must span a sensible width. With the max-advance bug the
    // clusters collapse onto each other and the rightmost origin stays well
    // under this bound; with the correct summed advance it clears it easily.
    // Measured: the max-advance bug collapses this word to ~64.8px, the correct
    // summed advance spreads it to ~85.5px. 75px sits between, so the test
    // fails on the bug and passes on the fix with margin either side.
    let rightmost = origins.iter().cloned().fold(0.0_f32, f32::max);
    assert!(
        rightmost > 75.0,
        "text collapsed — rightmost glyph origin was only {rightmost:.1}px \
         (indicates clusters advancing by max instead of sum)"
    );
}

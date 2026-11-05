//! Whitespace is an empty glyph, not a missing one.
//!
//! The space (U+0020) and the ideographic space (U+3000) draw nothing and advance the pen:
//! `generate` gives the empty field (no point inside), `skeleton` gives `Some` skeleton with no
//! strokes, and rasterising that skeleton gives the same glyph. `None` stays reserved for
//! characters without a glyph, which `generate` marks with its placeholder.
//!
//! Advances (oracle: typographic convention, not this crate's output): the word space is a
//! quarter of the em (U+2005 FOUR-PER-EM SPACE); the ideographic space is one em, the advance
//! of the full-width kana and kanji.
//!
//! Author: Moroya Sakamoto

use alice_font::glyph::{GlyphGenerator, GlyphSdf};
use alice_font::glyph_dispatcher::{generate, skeleton};
use alice_font::MetaFontParams;

const CASES: [(char, f32); 2] = [(' ', 0.25), ('\u{3000}', 1.0)];

fn params() -> [MetaFontParams; 3] {
    [
        MetaFontParams::sans_regular(),
        MetaFontParams::serif_italic(),
        MetaFontParams::display_heavy(),
    ]
}

fn blank(sdf: &GlyphSdf) -> bool {
    let empty = GlyphSdf::empty();
    sdf.data
        .iter()
        .zip(&empty.data)
        .all(|(a, b)| a.to_bits() == b.to_bits())
}

#[test]
fn whitespace_draws_nothing_and_advances() {
    for p in params() {
        for (ch, advance) in CASES {
            let g = generate(ch, &p);
            assert!(blank(&g), "{ch:?}: something is drawn");
            assert!(g.data.iter().all(|v| *v > 0.0), "{ch:?}: a point is inside");
            assert_eq!(
                g.advance.to_bits(),
                advance.to_bits(),
                "{ch:?}: advance {}",
                g.advance
            );
            // the box of nothing is the empty field's box (finite, min below max), not the
            // inverted extremes a stroke-less bounding box would give
            let e = GlyphSdf::empty();
            for (got, want) in [(g.bbox_min, e.bbox_min), (g.bbox_max, e.bbox_max)] {
                assert!(
                    got.x.is_finite() && got.y.is_finite(),
                    "{ch:?}: bbox {got:?}"
                );
                assert_eq!(
                    (got.x.to_bits(), got.y.to_bits()),
                    (want.x.to_bits(), want.y.to_bits()),
                    "{ch:?}"
                );
            }
        }
    }
}

#[test]
fn whitespace_has_an_empty_skeleton_not_none() {
    for p in params() {
        let gen = GlyphGenerator::new(&p);
        for (ch, advance) in CASES {
            let s = skeleton(ch, &p).unwrap_or_else(|| panic!("{ch:?}: None"));
            assert_eq!(s.stroke_count, 0, "{ch:?}");
            assert_eq!(s.advance.to_bits(), advance.to_bits(), "{ch:?}");
            let r = gen.generate_from_skeleton(&s);
            let g = generate(ch, &p);
            assert!(blank(&r) && blank(&g), "{ch:?}");
            assert_eq!(r.advance.to_bits(), g.advance.to_bits(), "{ch:?}");
        }
        // a character without a glyph is still None (the distinction this test is about)
        assert!(skeleton('€', &p).is_none());
    }
}

#[test]
fn whitespace_is_not_the_placeholder_box() {
    // the box generate draws for a byte without a glyph has points inside
    let p = MetaFontParams::sans_regular();
    let boxed = GlyphGenerator::new(&p).generate(0x7F);
    assert!(
        boxed.data.iter().any(|v| *v < 0.0),
        "the placeholder draws a box"
    );
    assert!(
        blank(&generate(' ', &p)),
        "the space draws the placeholder box"
    );
}

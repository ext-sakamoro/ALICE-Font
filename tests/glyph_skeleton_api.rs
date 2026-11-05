//! `glyph_dispatcher::skeleton`: the stroke skeleton of a character, for every script the
//! dispatcher draws.
//!
//! Oracle (parity with the existing rasterising entry): for every character, `skeleton(ch, p)`
//! is `Some(s)` exactly when `generate(ch, p)` draws a real glyph, and then rasterising `s`
//! with `GlyphGenerator::new(p).generate_from_skeleton(&s)` gives `generate(ch, p)` bit for
//! bit. Characters without a glyph give `None`; `generate` draws its placeholder for them
//! (one box for every unknown ASCII byte, the empty field for kana and kanji).
//!
//! Author: Moroya Sakamoto

use alice_font::cjk::ids_db::KANJI_DB;
use alice_font::glyph::{GlyphGenerator, GlyphSdf};
use alice_font::glyph_dispatcher::{self, generate, skeleton};
use alice_font::MetaFontParams;

fn same(a: &GlyphSdf, b: &GlyphSdf) -> bool {
    a.data
        .iter()
        .zip(&b.data)
        .all(|(x, y)| x.to_bits() == y.to_bits())
        && a.advance.to_bits() == b.advance.to_bits()
        && a.lsb.to_bits() == b.lsb.to_bits()
        && a.bbox_min.x.to_bits() == b.bbox_min.x.to_bits()
        && a.bbox_min.y.to_bits() == b.bbox_min.y.to_bits()
        && a.bbox_max.x.to_bits() == b.bbox_max.x.to_bits()
        && a.bbox_max.y.to_bits() == b.bbox_max.y.to_bits()
}

fn params() -> [MetaFontParams; 2] {
    [
        MetaFontParams::sans_regular(),
        MetaFontParams::serif_italic(),
    ]
}

/// Every character of a range the dispatcher routes, plus every registered kanji.
fn characters() -> Vec<char> {
    let mut v: Vec<char> = (0x20u32..=0x7E).filter_map(char::from_u32).collect();
    v.extend((0x3040u32..=0x30FF).filter_map(char::from_u32));
    v.extend(KANJI_DB.iter().map(|d| d.codepoint));
    // the ideographic space, an unregistered kanji and a character outside every range
    v.push('\u{3000}');
    v.push('鬱');
    v.push('€');
    v
}

#[test]
fn a_skeleton_rasterises_to_what_generate_draws() {
    let mut drawn = 0;
    for p in params() {
        let gen = GlyphGenerator::new(&p);
        for ch in characters() {
            if let Some(s) = skeleton(ch, &p) {
                // only whitespace has no strokes (tests/glyph_whitespace.rs)
                assert!(
                    s.stroke_count > 0 || ch == ' ' || ch == '\u{3000}',
                    "{ch:?}: a skeleton without strokes"
                );
                assert!(
                    same(&gen.generate_from_skeleton(&s), &generate(ch, &p)),
                    "{ch:?}: the skeleton does not rasterise to generate's glyph"
                );
                drawn += 1;
            }
        }
    }
    // A-Z, a-z, 0-9 at least, in both styles
    assert!(drawn >= 2 * 62, "only {drawn} skeletons");
}

#[test]
fn none_is_exactly_the_placeholder() {
    for p in params() {
        // coverage (spec): every printable ASCII byte has a glyph (the space an empty one,
        // tests/glyph_whitespace.rs), so no printable byte is None
        let ascii_none: Vec<char> = (0x20u32..=0x7E)
            .filter_map(char::from_u32)
            .filter(|&c| skeleton(c, &p).is_none())
            .collect();
        assert!(ascii_none.is_empty(), "{ascii_none:?}");
        // the placeholder itself: bytes outside the printable range drawn by the generator
        // (DEL and the control bytes) all get the same box, which has ink and is no real glyph
        let gen = GlyphGenerator::new(&p);
        let boxes: Vec<GlyphSdf> = [0x01u8, 0x1F, 0x7F, 0x80, 0xFF]
            .iter()
            .map(|&b| gen.generate(b))
            .collect();
        assert!(boxes.len() > 1);
        for w in boxes.windows(2) {
            assert!(same(&w[0], &w[1]), "the placeholder differs between bytes");
        }
        assert!(
            boxes[0].data.iter().any(|v| *v < 0.0),
            "the placeholder draws nothing"
        );
        for c in ('A'..='Z').chain('a'..='z').chain('0'..='9') {
            let s = skeleton(c, &p).unwrap_or_else(|| panic!("{c:?} has a glyph"));
            assert!(
                !same(&gen.generate_from_skeleton(&s), &boxes[0]),
                "{c:?} draws the placeholder box"
            );
        }
        // kana / kanji / other: None ⇔ the empty field, except whitespace (U+3000), which is
        // blank but has a glyph (an empty skeleton, tests/glyph_whitespace.rs)
        let empty = GlyphSdf::empty();
        for ch in characters().into_iter().filter(|c| (*c as u32) > 0x7E) {
            let none = skeleton(ch, &p).is_none();
            let blank = generate(ch, &p)
                .data
                .iter()
                .zip(&empty.data)
                .all(|(x, y)| x.to_bits() == y.to_bits());
            assert_eq!(none, blank && ch != '\u{3000}', "{ch:?}");
        }
        assert!(skeleton('鬱', &p).is_none());
        assert!(skeleton('€', &p).is_none());
    }
}

#[test]
fn every_registered_kanji_and_basic_kana_have_a_skeleton() {
    let p = MetaFontParams::sans_regular();
    for d in KANJI_DB {
        assert!(skeleton(d.codepoint, &p).is_some(), "{:?}", d.codepoint);
    }
    for c in "あいうえおアイウエオ".chars() {
        assert!(skeleton(c, &p).is_some(), "{c:?}");
    }
    assert_eq!(
        glyph_dispatcher::category('鬱'),
        glyph_dispatcher::GlyphCategory::CjkUnified
    );
}

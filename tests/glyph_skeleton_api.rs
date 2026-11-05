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
    a.data.iter().zip(&b.data).all(|(x, y)| x.to_bits() == y.to_bits())
        && a.advance.to_bits() == b.advance.to_bits()
        && a.lsb.to_bits() == b.lsb.to_bits()
        && a.bbox_min.x.to_bits() == b.bbox_min.x.to_bits()
        && a.bbox_min.y.to_bits() == b.bbox_min.y.to_bits()
        && a.bbox_max.x.to_bits() == b.bbox_max.x.to_bits()
        && a.bbox_max.y.to_bits() == b.bbox_max.y.to_bits()
}

fn params() -> [MetaFontParams; 2] {
    [MetaFontParams::sans_regular(), MetaFontParams::serif_italic()]
}

/// Every character of a range the dispatcher routes, plus every registered kanji.
fn characters() -> Vec<char> {
    let mut v: Vec<char> = (0x20u32..=0x7E).filter_map(char::from_u32).collect();
    v.extend((0x3040u32..=0x30FF).filter_map(char::from_u32));
    v.extend(KANJI_DB.iter().map(|d| d.codepoint));
    // an unregistered kanji and a character outside every range
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
                assert!(s.stroke_count > 0, "{ch:?}: a skeleton without strokes");
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
        // the ASCII placeholder (a box) is the same for every byte that has no glyph; '\x7F'
        // is outside the routed range, so take a byte the table leaves out: '~' if it has no
        // glyph, otherwise the first one found
        let ascii_none: Vec<char> = (0x20u32..=0x7E)
            .filter_map(char::from_u32)
            .filter(|&c| skeleton(c, &p).is_none())
            .collect();
        for w in ascii_none.windows(2) {
            assert!(same(&generate(w[0], &p), &generate(w[1], &p)), "{:?} vs {:?}", w[0], w[1]);
        }
        // the placeholder box is not any real glyph
        for c in ('A'..='Z').chain('a'..='z').chain('0'..='9') {
            let s = skeleton(c, &p).unwrap_or_else(|| panic!("{c:?} has a glyph"));
            if let Some(&n) = ascii_none.first() {
                assert!(!same(&GlyphGenerator::new(&p).generate_from_skeleton(&s), &generate(n, &p)));
            }
        }
        // kana / kanji / other: None ⇔ the empty field
        let empty = GlyphSdf::empty();
        for ch in characters().into_iter().filter(|c| (*c as u32) > 0x7E) {
            let none = skeleton(ch, &p).is_none();
            let blank = generate(ch, &p).data.iter().zip(&empty.data).all(|(x, y)| x.to_bits() == y.to_bits());
            assert_eq!(none, blank, "{ch:?}");
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
    assert_eq!(glyph_dispatcher::category('鬱'), glyph_dispatcher::GlyphCategory::CjkUnified);
}

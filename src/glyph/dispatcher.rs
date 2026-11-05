//! Glyph dispatcher — routes Unicode code points to the appropriate
//! parametric glyph generator (ASCII, hiragana, katakana, kanji, ...).
//!
//! All paths are procedural: the resulting SDF is derived from
//! parametric stroke skeletons (`stroke::Stroke`) shaped by
//! `MetaFontParams`, not from any embedded outline table or external
//! TTF. Coverage is bounded by which characters have parametric
//! skeleton definitions — see `cjk::ids_db` for CJK composition
//! coverage and the Roadmap for the S6 completion plan.
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use crate::glyph::{hiragana, kanji, katakana, GlyphGenerator, GlyphSdf, GlyphSkeleton};
use crate::param::MetaFontParams;

/// ASCII printable range (matches the existing `GlyphGenerator` coverage).
pub const ASCII_RANGE: (u32, u32) = (0x0020, 0x007E);

/// Hiragana code block (U+3040 to U+309F).
pub const HIRAGANA_RANGE: (u32, u32) = (0x3040, 0x309F);

/// Katakana code block (U+30A0 to U+30FF).
pub const KATAKANA_RANGE: (u32, u32) = (0x30A0, 0x30FF);

/// CJK Unified Ideographs block (U+4E00 to U+9FFF) — the Joyo kanji live here.
pub const CJK_UNIFIED_RANGE: (u32, u32) = (0x4E00, 0x9FFF);

/// Generate the SDF for an arbitrary character, routing by code-point
/// range to the appropriate parametric generator. Returns an empty
/// placeholder `GlyphSdf` (with `advance = 0.5`) when the character is
/// outside any generator's supported range.
#[must_use]
pub fn generate(ch: char, params: &MetaFontParams) -> GlyphSdf {
    let cp = ch as u32;
    if in_range(cp, ASCII_RANGE) {
        let gen = GlyphGenerator::new(params);
        gen.generate(ch as u8)
    } else if in_range(cp, HIRAGANA_RANGE) {
        hiragana::generate(ch, params)
    } else if in_range(cp, KATAKANA_RANGE) {
        katakana::generate(ch, params)
    } else if in_range(cp, CJK_UNIFIED_RANGE) {
        kanji::generate(ch, params)
    } else {
        let mut sdf = GlyphSdf::empty();
        sdf.advance = 0.5;
        sdf
    }
}

/// The stroke skeleton `generate` rasterises for `ch`, or `None` when it has no glyph for it
/// (it then draws a placeholder: a box for an ASCII byte, the empty field otherwise).
///
/// `GlyphGenerator::new(params).generate_from_skeleton(&s)` is `generate(ch, params)` bit for
/// bit. The skeleton is the law of the glyph (Bezier strokes in em units), so it can be
/// evaluated analytically instead of sampled (for example by `sdf_bridge` with the `sdf`
/// feature). `params` shapes the ASCII skeletons (heights, serifs); the kana and kanji
/// skeletons do not depend on it.
#[must_use]
pub fn skeleton(ch: char, params: &MetaFontParams) -> Option<GlyphSkeleton> {
    let cp = ch as u32;
    if in_range(cp, ASCII_RANGE) {
        GlyphGenerator::new(params).known_skeleton(ch as u8)
    } else if in_range(cp, HIRAGANA_RANGE) {
        hiragana::build_skeleton(ch)
    } else if in_range(cp, KATAKANA_RANGE) {
        katakana::build_skeleton(ch)
    } else if in_range(cp, CJK_UNIFIED_RANGE) {
        kanji::build_skeleton(ch)
    } else {
        None
    }
}

/// Returns the dispatch category for a code point, mostly useful for tests
/// and diagnostics.
#[must_use]
pub const fn category(ch: char) -> GlyphCategory {
    let cp = ch as u32;
    if cp >= ASCII_RANGE.0 && cp <= ASCII_RANGE.1 {
        GlyphCategory::Ascii
    } else if cp >= HIRAGANA_RANGE.0 && cp <= HIRAGANA_RANGE.1 {
        GlyphCategory::Hiragana
    } else if cp >= KATAKANA_RANGE.0 && cp <= KATAKANA_RANGE.1 {
        GlyphCategory::Katakana
    } else if cp >= CJK_UNIFIED_RANGE.0 && cp <= CJK_UNIFIED_RANGE.1 {
        GlyphCategory::CjkUnified
    } else {
        GlyphCategory::Unsupported
    }
}

/// Category of a code point as understood by the dispatcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphCategory {
    Ascii,
    Hiragana,
    Katakana,
    CjkUnified,
    Unsupported,
}

#[inline]
const fn in_range(cp: u32, range: (u32, u32)) -> bool {
    cp >= range.0 && cp <= range.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_ascii_letters() {
        assert_eq!(category('A'), GlyphCategory::Ascii);
        assert_eq!(category('z'), GlyphCategory::Ascii);
        assert_eq!(category('0'), GlyphCategory::Ascii);
        assert_eq!(category(' '), GlyphCategory::Ascii);
        assert_eq!(category('~'), GlyphCategory::Ascii);
    }

    #[test]
    fn category_hiragana_block() {
        assert_eq!(category('あ'), GlyphCategory::Hiragana);
        assert_eq!(category('ん'), GlyphCategory::Hiragana);
        assert_eq!(category('ゔ'), GlyphCategory::Hiragana);
    }

    #[test]
    fn category_katakana_block() {
        assert_eq!(category('ア'), GlyphCategory::Katakana);
        assert_eq!(category('ン'), GlyphCategory::Katakana);
        assert_eq!(category('ー'), GlyphCategory::Katakana);
    }

    #[test]
    fn category_cjk_unified() {
        assert_eq!(category('明'), GlyphCategory::CjkUnified);
        assert_eq!(category('字'), GlyphCategory::CjkUnified);
        assert_eq!(category('一'), GlyphCategory::CjkUnified);
    }

    #[test]
    fn category_unsupported() {
        // Tab is below ASCII printable range.
        assert_eq!(category('\t'), GlyphCategory::Unsupported);
        // Hangul is outside our currently-supported blocks.
        assert_eq!(category('가'), GlyphCategory::Unsupported);
    }

    #[test]
    fn ascii_routes_to_real_generator() {
        let sdf = generate('A', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
        // A real glyph has at least one inside pixel (negative SDF).
        let any_inside = sdf.data.iter().any(|d| *d < 0.0);
        assert!(any_inside, "ASCII glyph 'A' should have inside pixels");
    }

    #[test]
    fn hiragana_routes_to_stub() {
        let sdf = generate('あ', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
    }

    #[test]
    fn katakana_routes_to_stub() {
        let sdf = generate('ア', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
    }

    #[test]
    fn kanji_routes_to_stub() {
        let sdf = generate('明', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
    }

    #[test]
    fn unsupported_returns_placeholder_with_advance() {
        let sdf = generate('가', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
    }
}

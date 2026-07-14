//! Glyph dispatcher — routes Unicode code points to the appropriate
//! glyph generator (ASCII, hiragana, katakana, kanji, ...).
//!
//! This module is the single entry point that the atlas calls when it
//! needs to rasterize a glyph for an arbitrary `char`. It is part of the
//! CJK extension; see `docs/CJK_DESIGN.md` for the design rationale.
//!
//! License: MIT
//! Author: Moroya Sakamoto

use crate::glyph::{font_render, hiragana, kanji, katakana, GlyphGenerator, GlyphSdf};
use crate::param::MetaFontParams;

/// ASCII printable range (matches the existing `GlyphGenerator` coverage).
pub const ASCII_RANGE: (u32, u32) = (0x0020, 0x007E);

/// Hiragana code block (U+3040 to U+309F).
pub const HIRAGANA_RANGE: (u32, u32) = (0x3040, 0x309F);

/// Katakana code block (U+30A0 to U+30FF).
pub const KATAKANA_RANGE: (u32, u32) = (0x30A0, 0x30FF);

/// CJK Unified Ideographs block (U+4E00 to U+9FFF) — the Joyo kanji live here.
pub const CJK_UNIFIED_RANGE: (u32, u32) = (0x4E00, 0x9FFF);

/// Generate the SDF for an arbitrary character, routing by code-point range.
///
/// Returns a non-empty `GlyphSdf` for supported characters and an empty
/// placeholder (with `advance = 1.0`) for unsupported code points.
#[must_use]
pub fn generate(ch: char, params: &MetaFontParams) -> GlyphSdf {
    // First: BIZ UDPGothic outline table (highest quality, covers ASCII + CJK).
    if let Some(sdf) = font_render::rasterize(ch, params) {
        return sdf;
    }
    // Fallback: parametric skeleton (for chars not in the font table).
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

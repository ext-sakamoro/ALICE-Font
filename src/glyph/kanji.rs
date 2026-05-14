//! Kanji glyph rendering — U+4E00 to U+9FFF (CJK Unified Ideographs)
//!
//! Status: S2 stub. Full implementation is scheduled for S5 (composition
//! engine) and S6 (joyo kanji coverage). See `docs/CJK_KANJI_SPEC.md` for
//! the design specification.
//!
//! License: MIT
//! Author: Moroya Sakamoto

use crate::glyph::GlyphSdf;
use crate::param::MetaFontParams;

/// Generate SDF for a CJK Unified Ideograph.
///
/// In the S2 milestone this returns an empty placeholder glyph with a
/// reasonable advance. The real IDS-driven implementation lands in S5/S6.
#[must_use]
pub fn generate(_ch: char, _params: &MetaFontParams) -> GlyphSdf {
    let mut sdf = GlyphSdf::empty();
    sdf.advance = 1.0;
    sdf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_returns_empty_with_advance() {
        let sdf = generate('明', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
    }
}

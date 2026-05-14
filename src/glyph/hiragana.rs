//! Hiragana glyph rendering — U+3040 to U+309F
//!
//! Status: S2 stub. Full implementation is scheduled for S3.
//! See `docs/CJK_KANA_SPEC.md` for the design specification.
//!
//! License: MIT
//! Author: Moroya Sakamoto

use crate::glyph::GlyphSdf;
use crate::param::MetaFontParams;

/// Generate SDF for a hiragana character.
///
/// In the S2 milestone this returns an empty placeholder glyph with a
/// reasonable advance so layout code can already proceed. The real
/// stroke-composition implementation lands in S3.
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
        let sdf = generate('あ', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
    }
}

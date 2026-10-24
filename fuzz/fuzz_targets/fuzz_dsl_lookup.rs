//! Fuzz target: font_dsl::dsl_lookup(char) が任意 codepoint で panic しないことを検証
//!
//!
//! 起こり得る危険:
//! - u32 → char 変換で invalid Unicode scalar (surrogate 領域 0xD800..0xDFFF、
//!   > 0x10FFFF) は None であるべき
//! - 登録済 CJK char の SkeletonGlyph 生成で MAX_GLYPH_STROKES 境界の truncate
//!   が panic を起こさない
//! - stroke_count が MAX_GLYPH_STROKES を超えないこと

#![no_main]

use alice_font::font_dsl::dsl_lookup;
use alice_font::glyph::MAX_GLYPH_STROKES;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|codepoint: u32| {
    // u32 → char 変換、invalid scalar は fuzz 対象外 (Rust std の
    // char::from_u32 が None を返すのみで panic しない)
    let Some(ch) = char::from_u32(codepoint) else {
        return;
    };

    if let Some(glyph) = dsl_lookup(ch) {
        // 登録済 codepoint: SkeletonGlyph の invariant を検証
        assert_eq!(
            glyph.codepoint, ch,
            "dsl_lookup returned glyph with mismatched codepoint"
        );
        assert!(
            glyph.stroke_count <= MAX_GLYPH_STROKES,
            "stroke_count {} exceeded MAX_GLYPH_STROKES ({})",
            glyph.stroke_count,
            MAX_GLYPH_STROKES
        );
        assert!(
            glyph.pen_width.is_finite() && glyph.pen_width > 0.0,
            "pen_width invalid: {}",
            glyph.pen_width
        );
        assert!(glyph.advance.is_finite(), "advance not finite");
    }
});

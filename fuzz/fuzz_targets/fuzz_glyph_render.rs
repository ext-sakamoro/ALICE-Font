//! Fuzz target: GlyphGenerator::generate(u8) が任意 MetaFontParams + 任意 codepoint で
//! panic しないことを検証
//!
//!
//! 起こり得る危険:
//! - MetaFontParams の x_height / cap_height が 0 or 負値で NaN / div-by-zero
//! - 極端な slant で幾何演算が inf 生成 → assert 誤爆
//! - 未定義 codepoint (0x00, 0x80..) の placeholder 経路で panic

#![no_main]

use alice_font::{GlyphGenerator, MetaFontParams};
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    /// 40-byte MetaFontParams encoded form
    params_bytes: [u8; 40],
    /// arbitrary u8 codepoint (ASCII / punct / placeholder 分岐)
    ch: u8,
}

fuzz_target!(|input: FuzzInput| {
    // 非 finite field を含む packet は boundary で reject される (try_decode = None)
    // → 以降の NaN 伝播検証は finite params にのみ意味がある
    let Some(params) = MetaFontParams::try_decode(&input.params_bytes) else {
        return;
    };
    let gen = GlyphGenerator::new(&params);
    let sdf = gen.generate(input.ch);

    // sample が finite (not NaN / not Inf) であることを軽く検証
    // (0.5, 0.5) 中央 1 点のみでコスト最小、NaN 汚染を早期検出
    let v = sdf.sample(0.5, 0.5);
    assert!(
        v.is_finite() || v == f32::INFINITY || v == f32::NEG_INFINITY,
        "SDF sample produced NaN for ch=0x{:02X}",
        input.ch
    );
});

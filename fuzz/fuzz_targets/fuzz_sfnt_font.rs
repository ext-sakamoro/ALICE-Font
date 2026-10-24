//! Fuzz `alice_sfnt` against arbitrary font bytes.
//!
//! Input: 4 bytes of little-endian code point, then the font (the layout of the seeds in
//! fuzz/seeds/fuzz_sfnt_font/). Every face is opened, the code point is looked up (plain and with
//! VS1), and the glyph it maps to plus the first glyphs are drawn and measured. Every call returns a
//! Result: nothing may panic, in a debug build (cargo fuzz's default) or a release one.

#![no_main]

use alice_sfnt::{fonts_in, Contours, Font};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    let cp = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let font = &data[4..];
    let faces = fonts_in(font).unwrap_or(1).min(4);
    for index in 0..faces {
        let Ok(face) = Font::parse(font, index) else {
            continue;
        };
        let mapped = face.glyph_index(cp).ok().flatten();
        let _ = face.glyph_variation(cp, 0xFE00);
        let first = (0..face.num_glyphs()).take(32);
        for gid in mapped.into_iter().chain(first) {
            let _ = face.outline(gid, &mut Contours::default());
            let _ = face.advance(gid);
        }
    }
});

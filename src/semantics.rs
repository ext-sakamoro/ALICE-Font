//! Content hash of what this crate computes for fixed inputs: [`FONT_SEMANTICS_ID`] (ALICE-Law G6)
//!
//! The recorded-bit goldens in `tests/semantics_golden.rs` pin the bytes a fixed set of glyph computations
//! produces: the outline rasteriser's signed distance field, the parametric glyph generator's field, the pen
//! half-width over a sweep of stroke angles, the medial axis of a shape, the three shader payloads in their
//! std140 layout, and the styled raster of a glyph with a fixed effect stack. [`FONT_SEMANTICS_ID`] folds those
//! digests with the identifier of the one arithmetic crate every one of them is built on, alice-det-math's
//! `SEMANTICS_ID` (every transcendental and the fused-free rounding come from it). Two builds with the same
//! identifier compute the same bytes for every pinned scenario.
//!
//! # Hex form
//!
//! ```text
//! 210bcb70027cfb9415d15c7b752e2c0ee4350965ca8651285dc713af117366fd
//! ```
//!
//! # Fold
//!
//! The procedure of the `SEMANTICS_ID` fold of `alice-det-math`: sort the entries by name (ascending byte
//! order), then SHA-256 over, for each entry in that order, the name length as 4 bytes big-endian (`u32`), the
//! name bytes and the 32 digest bytes.
//!
//! # Independent of cargo features
//!
//! The table and the identifier are constants that no `cfg` gates, so a build with any feature set reports the
//! same identifier. The upstream entry is recorded as hex; `tests/font_semantics_id.rs` compares it with
//! alice-det-math's constant at the revision Cargo.toml pins.
//!
//! # Staleness
//!
//! [`FONT_SEMANTICS_CHECKED_AT`] names the crate version and toolchain the pins were last checked on; a
//! different version or toolchain fails `tests/font_semantics_id.rs` until they are checked again.

/// Decodes 64 hex digits into 32 bytes at compile time; anything else is a compile error.
const fn hex32(s: &str) -> [u8; 32] {
    const fn nibble(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            _ => panic!("not a lower-case hex digit"),
        }
    }
    let b = s.as_bytes();
    assert!(b.len() == 64, "a digest is 64 hex digits");
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = nibble(b[2 * i]) * 16 + nibble(b[2 * i + 1]);
        i += 1;
    }
    out
}

/// The entries [`FONT_SEMANTICS_ID`] folds: one per recorded-bit scenario of `tests/semantics_golden.rs`,
/// named as that file names it, and alice-det-math's identifier
pub const FONT_SEMANTICS_PINS: &[(&str, [u8; 32])] = &[
    (
        "raster sdf of a star over a box",
        hex32("ca784b9d637a6a132271d51ccb5ca4f30e0643ee5ca613222aeb49a9813f6dc7"),
    ),
    (
        "parametric glyph sdf",
        hex32("58b52eb6783ade233599dbfa6553430b05614d34230c11d61fc096f2530b5a01"),
    ),
    (
        "pen half width over stroke angles",
        hex32("2d080df3a5952ee5c7d6aefc4cff34dc0fb42f5249a5a0b21315ae5ccae11360"),
    ),
    (
        "medial axis of a star over a box",
        hex32("329c51132db22ce2869eb4088b4e616ffee2730db001d2bf47d930c29184d743"),
    ),
    (
        "analytical shader payload std140",
        hex32("1ab0f729b9234d9ababb18e8f51426febf208d155b45a2189f074602fec47f5a"),
    ),
    (
        "outline shader payload std140",
        hex32("89aceadec0312cda35ba3cd4ba8887b1adeed1e501e10b57db49b9127be7c6bc"),
    ),
    (
        "skeleton string payload std140",
        hex32("12bc9dac20827a3c9a86d1548d401f977ca844af2da73b3c73550d9a27770134"),
    ),
    (
        "styled glyph pixels",
        hex32("3eed7265a5fb213b46caa7c3f0fb4e0b5f4ea9beeff5f931ffa1d7e420329aec"),
    ),
    (
        "alice-det-math",
        hex32("85e0884623b154a3b2b264374907b6cbac09307ca3f9a08339e48c567d3cb315"),
    ),
];

/// The crate version and toolchain the pins were last checked against
pub const FONT_SEMANTICS_CHECKED_AT: (&str, &str) = ("0.7.6", "1.98.1");

/// Content hash of what this crate computes for fixed inputs: the fold of [`FONT_SEMANTICS_PINS`]
/// described in the [module documentation](self)
pub const FONT_SEMANTICS_ID: [u8; 32] =
    hex32("210bcb70027cfb9415d15c7b752e2c0ee4350965ca8651285dc713af117366fd");

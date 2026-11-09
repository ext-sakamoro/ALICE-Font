//! Content hash of what this crate computes for fixed inputs: [`FONT_SEMANTICS_ID`] (ALICE-Law G6)
//!
//! The recorded-bit goldens in `tests/semantics_golden.rs` pin the bytes a fixed set of glyph computations
//! produces: the outline rasteriser's signed distance field, the parametric glyph generator's field, the pen
//! half-width over a sweep of stroke angles, the medial axis of a shape, the three shader payloads in their
//! std140 layout, and the styled raster of a glyph with a fixed effect stack. [`FONT_SEMANTICS_ID`] folds those
//! digests with [`DET_MATH_PINS`], the identifier of each alice-det-math function this crate calls
//! ([`DET_MATH_FUNCTIONS`]). Two builds with the same identifier compute the same bytes for every pinned
//! scenario. A function of alice-det-math this crate does not call moves nothing; one it calls stops the compile.
//!
//! # Hex form
//!
//! ```text
//! 3c75cddd32266ff4e3a2ca00e15883197de3ed8c4f17f8ab0ddb1d22c40d1232
//! ```
//!
//! # Fold
//!
//! The procedure of the `SEMANTICS_ID` fold of `alice-det-math`: sort the entries by name (ascending byte
//! order), then SHA-256 over, for each entry in that order, the name length as 4 bytes big-endian (`u32`), the
//! name bytes and the 32 digest bytes. A [`DET_MATH_PINS`] entry goes in under the name `det-math <function>`.
//!
//! # Independent of cargo features
//!
//! The table and the identifier are constants that no `cfg` gates, so a build with any feature set reports the
//! same identifier. Each [`DET_MATH_PINS`] entry is compared at compile time with `alice_det_math::function_pin`
//! at the commit the build resolves.
//!
//! # Staleness
//!
//! [`FONT_SEMANTICS_CHECKED_AT`] names the crate version and toolchain the pins were last checked on; a
//! different version or toolchain fails `tests/font_semantics_id.rs` until they are checked again.

/// Decodes 64 hex digits into 32 bytes at compile time; anything else is a compile error.
pub(crate) const fn hex32(s: &str) -> [u8; 32] {
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
/// named as that file names it
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
];

/// The functions of `alice-det-math` this crate calls (ALICE-Law's `scripts/fold_set_check.py` compares this
/// with the calls in the source and writes it)
pub const DET_MATH_FUNCTIONS: &[&str] = &[
    "argmax",
    "argmin",
    "clamp64",
    "max",
    "max64",
    "min",
    "min64",
    "ops_greatest",
    "ops_into_inner",
    "ops_least",
    "ops_offer",
    "round",
    "round64",
    "sqrt",
    "sqrt64",
];

/// The identifier of each function of [`DET_MATH_FUNCTIONS`], in that order, as recorded when
/// [`FONT_SEMANTICS_ID`] was computed. A function whose identifier moves in the resolved `alice-det-math` stops
/// the compile, rather than reporting an identifier computed from other bits.
pub const DET_MATH_PINS: &[(&str, [u8; 32])] = &[
    (
        "argmax",
        hex32("be0e7dd2e41e862b02d270b76df8f38fe185ef18af44e37176edfbf634f50c92"),
    ),
    (
        "argmin",
        hex32("64cfc0d660c21c8f8bbc084386a2bcbb2fb53e735ddf4a6e412bdb102c2025e6"),
    ),
    (
        "clamp64",
        hex32("9f865dc2c1d92f4f286c7dd6ce5ea1e6a3621a38e90d662c848626b7bed213fa"),
    ),
    (
        "max",
        hex32("8bd32fcb313f7fb3d7a699fc3075273de7e5a4e083912d8dc7f4cc4016fc2286"),
    ),
    (
        "max64",
        hex32("aac586eacc9acd69cf5988ffe736283c71fad8146027e86034d3e9b881db28b6"),
    ),
    (
        "min",
        hex32("2939e83813b26c868ec3113fb1c25102f16f28d8f24c343a727d4b1bec5904fe"),
    ),
    (
        "min64",
        hex32("1bef418073152f5e1e76f41718aa29b2517de8966a8c8ea8c1c2b5ba92fab3cb"),
    ),
    (
        "ops_greatest",
        hex32("2aba9f45e20e03407c691111f2515e607b9c53e65c05056ccfde5cbfe12b25f9"),
    ),
    (
        "ops_into_inner",
        hex32("e6da57a23db610a0bc5a24d28feb951f48f327404abe3f72df39254bd0b9852b"),
    ),
    (
        "ops_least",
        hex32("a9643ac20a8bebe65e0ab6e3fe2db56b6a1ed7f8f391126c32ad1877182c1be3"),
    ),
    (
        "ops_offer",
        hex32("39b3be3732db0ad0bec9c7af20d99de1a4a6e52f0ddf4f5b268ca74b16ff060e"),
    ),
    (
        "round",
        hex32("664695a5ceabc94e9fd1f6fb956055e1e3c1e2eab35c6af53ab97881e495bb0a"),
    ),
    (
        "round64",
        hex32("404a18a94c2afdb620c4d08b2431bbf7dfe4e8d40e9e161bbdda47ed895ab309"),
    ),
    (
        "sqrt",
        hex32("7312a1b9e6b3af0b13b84a71c447c05e013d3012db4aefd859808ef0453ec96d"),
    ),
    (
        "sqrt64",
        hex32("051b24b99798891af42895e889ecb5add515496fc771ba5637a830f8f645988d"),
    ),
];

const _: () = alice_det_math::assert_function_pins(DET_MATH_PINS);

/// The crate version and toolchain the pins were last checked against
pub const FONT_SEMANTICS_CHECKED_AT: (&str, &str) = ("0.8.0", "1.98.1");

/// Content hash of what this crate computes for fixed inputs: the fold of [`FONT_SEMANTICS_PINS`]
/// described in the [module documentation](self)
pub const FONT_SEMANTICS_ID: [u8; 32] =
    hex32("3c75cddd32266ff4e3a2ca00e15883197de3ed8c4f17f8ab0ddb1d22c40d1232");

/// The values [`FONT_SEMANTICS_ID`] had before: a stamped form under one of them was computed to the same bytes
/// this build computes, so [`crate::law_stamp::read`] reads it. `210bcb70…` folded alice-det-math's whole
/// identifier; the eight recorded outputs are the same under it.
pub const PREVIOUS_SEMANTICS_IDS: &[[u8; 32]] = &[hex32(
    "210bcb70027cfb9415d15c7b752e2c0ee4350965ca8651285dc713af117366fd",
)];

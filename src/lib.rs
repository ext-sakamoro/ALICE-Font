//! # ALICE-Font
//!
//! **Procedural CJK font — no atlas, no TTF, no bitmap.**
//!
//! ALICE-Font follows the ALICE principle 「データを送るな、法則を送れ」
//! (don't ship data — ship laws). The crate carries no font-file
//! outline table, no pre-baked SDF atlas, and no bitmap. Every glyph
//! is composed **procedurally at runtime** from three primitive
//! sources:
//!
//! | Primitive | Payload | Role |
//! |-----------|---------|------|
//! | [`MetaFontParams`] | **40 bytes** (10 × f32) | Style: weight, width, serif, contrast, slant, metrics, roundness |
//! | Radical stroke library ([`glyph::cjk_strokes`], [`cjk::radicals`]) | ~150 KB of Rust const Bezier skeletons | Shape of each of the 214 Kang Xi radicals |
//! | IDS composition rules ([`cjk::ids_db`]) | ~30 KB when complete | How each kanji decomposes into radicals |
//!
//! At runtime the parametric pen ([`stroke::PenModel`]) walks each
//! radical's Bezier skeleton with a width and serif profile derived
//! from `MetaFontParams`, `cjk::layout` places the radicals per the
//! IDS operator (⿰⿱⿲⿴…), and the union becomes the glyph SDF.
//! No lookup into a texture, no dependence on `.ttf`/`.otf`, no
//! embedded outline table.
//!
//! ## Current coverage & Roadmap
//!
//! Coverage today (v0.4.0):
//! - ASCII printable (0x20–0x7E) — full parametric
//! - Hiragana / Katakana — full parametric
//! - CJK Unified Ideographs — the subset registered in [`cjk::ids_db`]
//!   (~100 kanji as of writing)
//!
//! ## Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`param`] | `MetaFontParams` — 40-byte parametric font descriptor (10 × f32) |
//! | [`stroke`] | Variable-width pen model applied along Bezier skeleton curves |
//! | [`glyph`] | SDF glyph generation — upper/lower/digit/punct + CJK stroke primitives |
//! | [`atlas`] | GPU-friendly SDF texture atlas with LRU eviction |
//! | [`shaper`] | Text shaper — kerning, horizontal advance, line layout |
//! | [`license`] | Font license tracking — per-title, platform, seat-limit (32-byte wire format) |
//! | [`game`] | Game text effects — outline, shadow, glow, gradient via SDF compositing |
//!
//! ## Quick Start
//!
//! ```
//! use alice_font::{MetaFontParams, GlyphGenerator};
//!
//! // 40-byte sans-regular preset
//! let params = MetaFontParams::sans_regular();
//!
//! // Generate SDF glyph for 'A'
//! let gen = GlyphGenerator::new(&params);
//! let glyph = gen.generate(b'A');
//!
//! // Sample distance at center — negative = inside the glyph
//! let dist = glyph.sample(0.5, 0.5);
//! assert!(dist.is_finite());
//! ```
//!
//! ## Presets
//!
//! | Preset | Weight | Contrast | Serif |
//! |--------|--------|----------|-------|
//! | `sans_regular` | 0.08 | 0.0 | 0.0 |
//! | `sans_bold` | 0.14 | 0.0 | 0.0 |
//! | `serif_regular` | 0.08 | 0.4 | 0.03 |
//! | `serif_italic` | 0.07 | 0.3 | 0.025 (14° italic) |
//! | `mono_regular` | 0.08 | 0.0 | 0.0 (fixed advance) |
//! | `display_heavy` | 0.20 | 0.0 | 0.0 |
//!
//! Author: Moroya Sakamoto

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::module_name_repetitions,
    clippy::inline_always,
    clippy::too_many_lines
)]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

pub mod atlas;
pub mod bidi;
pub mod cjk;
pub mod composite;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod font_dsl;
pub mod game;
pub mod glyph;
pub mod license;
pub mod ligature;
pub mod medial_axis;
pub mod outline;
pub mod param;
pub mod prelude;
#[cfg(feature = "pyo3")]
pub mod python;
pub mod shader;
pub mod shaper;
pub mod skeleton_shader;
pub mod stroke;

/// ALICE-SDF bridge (Milestone A.3): Stroke / `GlyphSkeleton` → `alice_sdf::Sdf2dNode` parametric adapter
#[cfg(feature = "sdf")]
pub mod sdf_bridge;

pub use atlas::{AtlasEntry, AtlasEntryMulti, SdfAtlas, SdfAtlasMulti, SdfAtlasPage};
pub use game::{Color4, EffectStack, GameTextStyle, StyledGlyph, TextEffect};
pub use glyph::cjk_strokes::{add_cjk_stroke, CjkStrokeType, StrokePlacement};
pub use glyph::dispatcher::{self as glyph_dispatcher, GlyphCategory};
pub use glyph::helpers::KAPPA;
pub use glyph::{GlyphGenerator, GlyphSdf};
pub use license::{
    FontLicense, LicenseType, LicenseValidator, PlatformRestriction, UsageRights, ValidationResult,
};
pub use param::MetaFontParams;
pub use shaper::TextShaper;
pub use stroke::{PenModel, Stroke};

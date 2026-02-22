//! # ALICE-Font
//!
//! **Parametric Metafont Renderer — 40 bytes replace multi-MB font files.**
//!
//! ALICE-Font procedurally generates infinite-resolution SDF glyphs from
//! a 10-dimensional parameter space. Weight, width, serif style, and italic
//! angle are all continuous — interpolate between any two typefaces at runtime.
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

#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

pub mod atlas;
pub mod game;
pub mod glyph;
pub mod license;
pub mod param;
pub mod shaper;
pub mod stroke;

pub use atlas::SdfAtlas;
pub use game::{Color4, EffectStack, GameTextStyle, StyledGlyph, TextEffect};
pub use glyph::cjk_strokes::{add_cjk_stroke, CjkStrokeType, StrokePlacement};
pub use glyph::helpers::KAPPA;
pub use glyph::{GlyphGenerator, GlyphSdf};
pub use license::{
    FontLicense, LicenseType, LicenseValidator, PlatformRestriction, UsageRights, ValidationResult,
};
pub use param::MetaFontParams;
pub use shaper::TextShaper;
pub use stroke::{PenModel, Stroke};

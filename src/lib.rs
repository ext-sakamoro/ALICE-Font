//! ALICE-Font — Parametric metafont renderer
//!
//! Replaces multi-MB font files with 40-byte parameters that
//! procedurally generate infinite-resolution SDF glyphs.
//!
//! License: MIT
//! Author: Moroya Sakamoto

#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

pub mod param;
pub mod stroke;
pub mod glyph;
pub mod atlas;
pub mod shaper;

pub use param::MetaFontParams;
pub use stroke::{Stroke, PenModel};
pub use glyph::{GlyphGenerator, GlyphSdf};
pub use atlas::SdfAtlas;
pub use shaper::TextShaper;

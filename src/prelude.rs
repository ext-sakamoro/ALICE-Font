//! Convenience re-export (= `use alice_font::prelude::*;` で主要 API 一括取得)
//!
//! Font system 13 module のうち core (atlas / game / glyph / license / param /
//! shaper / stroke + glyph 系 sub-module) の主要型を prelude で提供
//! `bidi` / `cjk` / `composite` / `ffi` / `ligature` / `python` は補助 / feature

pub use crate::atlas::{AtlasEntry, AtlasEntryMulti, SdfAtlas, SdfAtlasMulti, SdfAtlasPage};
pub use crate::game::{Color4, EffectStack, GameTextStyle, StyledGlyph, TextEffect};
pub use crate::glyph::cjk_strokes::{add_cjk_stroke, CjkStrokeType, StrokePlacement};
pub use crate::glyph::dispatcher::{self as glyph_dispatcher, GlyphCategory};
pub use crate::glyph::helpers::KAPPA;
pub use crate::glyph::{GlyphGenerator, GlyphSdf};
pub use crate::license::{
    FontLicense, LicenseType, LicenseValidator, PlatformRestriction, UsageRights, ValidationResult,
};
pub use crate::param::MetaFontParams;
pub use crate::shaper::TextShaper;
pub use crate::stroke::{PenModel, Stroke};

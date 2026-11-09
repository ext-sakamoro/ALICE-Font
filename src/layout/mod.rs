//! Paragraph layout: line breaking with Japanese kinsoku (JIS X 4051) and Latin word breaks,
//! pen advances from a font's metrics, in integer font units.
//!
//! Every quantity is an `i32` in the font units of the metrics (`units_per_em`), so the same
//! input gives the same bits on every target (native and wasm32). Advances come from a
//! [`GlyphMetrics`]: a bezier font gives its `hmtx` advances as they are (integers); the
//! procedural glyphs of this crate give theirs through [`ProceduralMetrics`], which rounds the
//! em advance once with `alice_det_math::ops::round`.
//!
//! A paragraph is one block of plain text (no markup) and a set of style spans covering it with
//! no gap or overlap; each span names the [`StyleId`] whose metrics its characters use. `src`
//! offsets are UTF-8 byte offsets into that text.
//!
//! Rules (see [`jis`]): no line starts with a character JIS X 4051 keeps off the start of a line
//! or ends with one it keeps off the end, two of the same inseparable character are not split,
//! and a Latin word is not split. When no allowed break fits, the line breaks before the
//! character that overflows and is marked [`LineEnd::Forced`]. Lines are never stretched:
//! justification is not part of this layout.

pub mod jis;

pub use jis::{BreakClass, Kinsoku};

use crate::glyph::dispatcher;
use crate::param::MetaFontParams;
use alloc::vec::Vec;
use core::ops::Range;

/// The pen advances of one font at one style, in its own font units.
pub trait GlyphMetrics {
    /// Font units per em
    fn units_per_em(&self) -> u16;
    /// The advance of `ch`, `None` when the font has no glyph for it
    fn advance(&self, ch: char) -> Option<i32>;
    /// The kerning between `left` and `right` (added to `left`'s advance)
    fn kern(&self, left: char, right: char) -> i32 {
        let _ = (left, right);
        0
    }
}

/// A style of a paragraph (regular, strong, code, ...), resolved by a [`MetricsSet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StyleId(pub u16);

/// The metrics of each style a paragraph uses. Every style shares one `units_per_em`.
pub trait MetricsSet {
    /// The metrics of `style`, `None` when the set has no such style
    fn get(&self, style: StyleId) -> Option<&dyn GlyphMetrics>;
}

/// The characters of `src` (a byte range of the text) are set in `style`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StyleSpan {
    pub src: Range<u32>,
    pub style: StyleId,
}

/// The parameters of one paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutParams {
    /// The line measure (the most a line's content may be), font units, > 0
    pub measure: i32,
    /// The distance from one baseline to the next, font units, > 0
    pub line_height: i32,
    /// The baseline's distance below the top of its line, font units, 0..=line_height
    pub ascent: i32,
    /// How strictly the start of a line is guarded
    pub kinsoku: Kinsoku,
    /// Let a full stop or comma (cl-06 / cl-07) that would overflow hang past the measure
    pub hanging_punct: bool,
}

/// One glyph placed on a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlacedGlyph {
    pub ch: char,
    /// UTF-8 byte offset of `ch` in the text
    pub src: u32,
    pub style: StyleId,
    /// Pen position from the start of the line, font units
    pub x: i32,
    /// The advance this glyph moves the pen by (its kerning with the next glyph included)
    pub advance: i32,
}

/// Why a line ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineEnd {
    /// A newline in the text (the newline is not a glyph)
    Hard,
    /// An allowed break
    Soft,
    /// No allowed break fitted: broken before the glyph that overflowed
    Forced,
    /// The end of the text
    End,
}

/// One line of a paragraph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Line {
    /// Its glyphs, as a range of [`Paragraph::glyphs`]
    pub glyphs: Range<u32>,
    /// The bytes of the text it holds (a newline that ended it is not included)
    pub src: Range<u32>,
    /// The content's width: the advances of its glyphs except trailing spaces and a hung
    /// mark; never more than the measure
    pub width: i32,
    /// The width of the full stop or comma hung past the measure, 0 when none
    pub hang: i32,
    /// The baseline's distance from the top of the paragraph: `index * line_height + ascent`
    pub baseline: i32,
    pub end: LineEnd,
}

/// A laid-out paragraph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Paragraph {
    pub glyphs: Vec<PlacedGlyph>,
    pub lines: Vec<Line>,
    /// `lines.len() * line_height`
    pub height: i32,
    pub units_per_em: u16,
}

/// What [`layout`] refuses.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LayoutError {
    /// The style of the span holding `ch` has no glyph for it
    UnknownGlyph { ch: char, src: u32 },
    /// A single glyph is wider than the measure
    MeasureTooSmall { needed: i32, src: u32 },
    /// The parameters or the spans are malformed (the message says how)
    BadParams(&'static str),
}

impl core::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownGlyph { ch, src } => {
                write!(f, "layout: no glyph for {ch:?} at byte {src}")
            }
            Self::MeasureTooSmall { needed, src } => {
                write!(
                    f,
                    "layout: the glyph at byte {src} needs {needed} units, more than the measure"
                )
            }
            Self::BadParams(why) => write!(f, "layout: {why}"),
        }
    }
}

/// Lays out `text` in lines of at most `params.measure`.
///
/// # Errors
/// [`LayoutError::BadParams`] for a measure or line height <= 0, an ascent outside
/// `0..=line_height`, spans that do not cover the
/// text exactly (gap, overlap, unsorted, off a character boundary, past the end), a style the
/// set lacks, or styles with different `units_per_em`; [`LayoutError::UnknownGlyph`] and
/// [`LayoutError::MeasureTooSmall`] as named.
pub fn layout(
    text: &str,
    spans: &[StyleSpan],
    metrics: &dyn MetricsSet,
    params: &LayoutParams,
) -> Result<Paragraph, LayoutError> {
    let _ = (text, spans, metrics, params);
    todo!("STUB: paragraph layout (the oracles in tests/layout_oracle.rs come first)")
}

/// The procedural glyphs of this crate as [`GlyphMetrics`]: the em advance of each glyph's
/// skeleton, rounded once to `units_per_em` units (ties away from zero, the same bits on
/// every target).
pub struct ProceduralMetrics {
    pub params: MetaFontParams,
    pub units_per_em: u16,
}

impl GlyphMetrics for ProceduralMetrics {
    fn units_per_em(&self) -> u16 {
        self.units_per_em
    }
    fn advance(&self, ch: char) -> Option<i32> {
        let s = dispatcher::skeleton(ch, &self.params)?;
        let units = alice_det_math::ops::round(s.advance * f32::from(self.units_per_em));
        // an em advance is a few em at most, so the rounded units fit an i32
        #[allow(clippy::cast_possible_truncation)]
        Some(units as i32)
    }
}

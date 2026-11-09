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

/// The recorded digests [`LAYOUT_SEMANTICS_ID`] folds (`tests/layout_semantics_id.rs` recomputes each):
/// `"jis class table"`, the SHA-256 over every code point (`u32` little-endian) and its
/// [`BreakClass`] number (`u8`) for the code points whose class is not `Other`; and
/// `"layout corpus"`, the corpus digest of `tests/layout_oracle.rs`
/// (`the_corpus_layout_is_the_recorded_bits`); and `"layout corpus, no wrap"`, the same corpus under
/// [`Wrap::None`] (`the_no_wrap_corpus_layout_is_the_recorded_bits`). Any change to the classes or
/// to line breaking moves one.
pub const LAYOUT_SEMANTICS_PINS: &[(&str, [u8; 32])] = &[
    (
        "jis class table",
        crate::semantics::hex32("334f3e0cff14e9156aa5fb0ea6a5d52bd116b3d5dff1d9ed9b2525767dad9ea6"),
    ),
    (
        "layout corpus",
        crate::semantics::hex32("f932be5b592c021efc7266266c814cd623c17f74f8d263b0d69ff57ff837bfee"),
    ),
    (
        "layout corpus, no wrap",
        crate::semantics::hex32("156bb1b4e0ec735eda95c5816598979f513383ea49bed5b357d86158fdf0a8e8"),
    ),
];

/// The identifier of this layout's arithmetic: the fold of [`LAYOUT_SEMANTICS_PINS`] by the
/// procedure of [`crate::semantics`] (sort by name; SHA-256 over the name length as `u32`
/// big-endian, the name and the digest). It is separate from
/// [`crate::semantics::FONT_SEMANTICS_ID`], so a change to line breaking does not move the glyph
/// identifier; a reader that persists laid-out lines carries this one.
pub const LAYOUT_SEMANTICS_ID: [u8; 32] =
    crate::semantics::hex32("8f88a50060487b5280f6e144548f65b16b5a90bac4fe6d2a24bb893f855dcf2f");

/// Earlier values of [`LAYOUT_SEMANTICS_ID`] whose builds lay out every scenario they pinned to the
/// same bits as this one. `b832f7ef…` pinned the class table and the wrapped corpus (both unchanged
/// here); it had no `Wrap::None`, which only adds a pin.
pub const LAYOUT_PREVIOUS_SEMANTICS_IDS: &[[u8; 32]] = &[crate::semantics::hex32(
    "b832f7ef7acc942e36be83d8a9ef872a470f9bc56e66cf4d86fc45d84610e835",
)];

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
    /// Whether long lines break: `Soft` breaks them within the measure; `None` breaks only at
    /// a newline (a code block), and a line wider than the measure reports its `overflow`
    pub wrap: Wrap,
}

/// How a paragraph's lines break.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wrap {
    /// Break within the measure (kinsoku, Latin words, hanging, forced breaks)
    Soft,
    /// Break only at a newline; a line may be wider than the measure
    None,
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
    /// How far the content runs past the measure (`width - measure`), 0 when it fits; only
    /// [`Wrap::None`] lets a line overflow
    pub overflow: i32,
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
    if params.measure <= 0 || params.line_height <= 0 {
        return Err(LayoutError::BadParams(
            "measure and line_height must be > 0",
        ));
    }
    if params.ascent < 0 || params.ascent > params.line_height {
        return Err(LayoutError::BadParams(
            "ascent must be within 0..=line_height",
        ));
    }
    let len = u32::try_from(text.len()).map_err(|_| LayoutError::BadParams("text over 4 GiB"))?;
    let upm = check_spans(text, len, spans, metrics)?;
    let chars = shape(text, spans, metrics)?;

    let mut glyphs: Vec<PlacedGlyph> = Vec::with_capacity(chars.len());
    let mut lines: Vec<Line> = Vec::new();
    // each segment between newlines is broken on its own; a newline ends a line (Hard)
    let mut seg_start = 0usize;
    while seg_start < chars.len() {
        let mut seg_end = seg_start;
        while seg_end < chars.len() && chars[seg_end].ch != '\n' {
            seg_end += 1;
        }
        let hard = seg_end < chars.len();
        let at = chars.get(seg_start).map_or(len, |c| c.src);
        break_segment(
            &chars[seg_start..seg_end],
            at,
            hard,
            params,
            &mut glyphs,
            &mut lines,
        )?;
        seg_start = seg_end + 1;
        if hard && seg_start == chars.len() {
            // a text that ends in a newline ends with an empty line
            let at = chars[seg_end].src + 1;
            push_line(
                &mut lines,
                &glyphs,
                glyphs.len(),
                at..at,
                0,
                0,
                LineEnd::End,
                params,
            );
        }
    }
    let n = i32::try_from(lines.len()).map_err(|_| LayoutError::BadParams("too many lines"))?;
    Ok(Paragraph {
        glyphs,
        lines,
        height: n * params.line_height,
        units_per_em: upm,
    })
}

/// One character of the text with its style and its advance (kerning with the next character
/// of the same style included; a line end drops it again).
struct Shaped {
    ch: char,
    src: u32,
    style: StyleId,
    advance: i32,
    kern: i32,
}

fn check_spans(
    text: &str,
    len: u32,
    spans: &[StyleSpan],
    metrics: &dyn MetricsSet,
) -> Result<u16, LayoutError> {
    let bad = LayoutError::BadParams;
    if text.is_empty() {
        return if spans.is_empty() {
            Ok(0)
        } else {
            Err(bad("spans on an empty text"))
        };
    }
    let mut at = 0u32;
    let mut upm = None;
    for s in spans {
        if s.src.start != at {
            return Err(bad(
                "spans must cover the text in order, with no gap or overlap",
            ));
        }
        if s.src.end <= s.src.start || s.src.end > len {
            return Err(bad("a span is empty or runs past the end of the text"));
        }
        if !text.is_char_boundary(s.src.end as usize) {
            return Err(bad("a span ends off a character boundary"));
        }
        let m = metrics
            .get(s.style)
            .ok_or(bad("a span names a style the metrics set lacks"))?;
        if *upm.get_or_insert(m.units_per_em()) != m.units_per_em() {
            return Err(bad("the styles of a paragraph must share units_per_em"));
        }
        at = s.src.end;
    }
    if at != len {
        return Err(bad("the spans do not reach the end of the text"));
    }
    upm.ok_or(bad("no span"))
}

fn shape(
    text: &str,
    spans: &[StyleSpan],
    metrics: &dyn MetricsSet,
) -> Result<Vec<Shaped>, LayoutError> {
    let mut out: Vec<Shaped> = Vec::new();
    let mut span = 0usize;
    for (i, ch) in text.char_indices() {
        // check_spans bounded the text by u32
        #[allow(clippy::cast_possible_truncation)]
        let src = i as u32;
        while spans[span].src.end <= src {
            span += 1;
        }
        let style = spans[span].style;
        let advance = if ch == '\n' {
            0
        } else {
            let m = metrics
                .get(style)
                .ok_or(LayoutError::BadParams("unknown style"))?;
            m.advance(ch).ok_or(LayoutError::UnknownGlyph { ch, src })?
        };
        if let Some(prev) = out.last_mut() {
            if prev.style == style && prev.ch != '\n' && ch != '\n' {
                let m = metrics
                    .get(style)
                    .ok_or(LayoutError::BadParams("unknown style"))?;
                prev.kern = m.kern(prev.ch, ch);
            }
        }
        out.push(Shaped {
            ch,
            src,
            style,
            advance,
            kern: 0,
        });
    }
    Ok(out)
}

/// Breaks one segment (no newline inside) into lines, appending its glyphs and lines.
fn break_segment(
    seg: &[Shaped],
    at: u32,
    hard: bool,
    p: &LayoutParams,
    glyphs: &mut Vec<PlacedGlyph>,
    lines: &mut Vec<Line>,
) -> Result<(), LayoutError> {
    if seg.is_empty() {
        // an empty segment before a newline is an empty line
        // (`at` is the newline's offset)
        if hard {
            push_line(lines, glyphs, glyphs.len(), at..at, 0, 0, LineEnd::Hard, p);
        }
        return Ok(());
    }
    let mut start = 0usize;
    while start < seg.len() {
        let (end, hang, why) = fit(seg, start, p)?;
        let first = glyphs.len();
        let mut x = 0i32;
        for (k, s) in seg[start..end].iter().enumerate() {
            // the last glyph of a line has no kerning with the next line's first glyph
            let advance = if start + k + 1 == end {
                s.advance
            } else {
                s.advance + s.kern
            };
            glyphs.push(PlacedGlyph {
                ch: s.ch,
                src: s.src,
                style: s.style,
                x,
                advance,
            });
            x += advance;
        }
        let trailing: i32 = glyphs[first..]
            .iter()
            .rev()
            .take_while(|g| g.ch == ' ')
            .map(|g| g.advance)
            .sum();
        let last = &seg[end - 1];
        let src = seg[start].src..last.src + last.ch.len_utf8() as u32;
        let end_kind = match (why, end == seg.len()) {
            (Some(w), _) => w,
            (None, true) if hard => LineEnd::Hard,
            (None, _) => LineEnd::End,
        };
        push_line(
            lines,
            glyphs,
            first,
            src,
            x - trailing - hang,
            hang,
            end_kind,
            p,
        );
        start = end;
    }
    Ok(())
}

/// The end of the line that starts at `start`: (one past its last glyph, the hung mark's width,
/// `Some(Soft | Forced)` when it breaks before the end of the segment).
fn fit(
    seg: &[Shaped],
    start: usize,
    p: &LayoutParams,
) -> Result<(usize, i32, Option<LineEnd>), LayoutError> {
    if p.wrap == Wrap::None {
        // a segment holds no newline, so without wrapping it is one line
        return Ok((seg.len(), 0, None));
    }
    let mut x = 0i32; // pen after the glyphs so far (kerning between them included)
    let mut last_break: Option<usize> = None;
    for i in start..seg.len() {
        let s = &seg[i];
        if i > start && jis::break_allowed(seg[i - 1].ch, s.ch, p.kinsoku) {
            last_break = Some(i);
        }
        if s.ch == ' ' {
            // a space never overflows: at the end of a line it takes no width
            x += s.advance + s.kern;
            continue;
        }
        let content = x + s.advance; // this glyph ends the content; its kerning is not counted
        if content > p.measure {
            let mark = matches!(jis::class(s.ch), BreakClass::FullStop | BreakClass::Comma);
            let next_ok = i + 1 == seg.len() || jis::break_allowed(s.ch, seg[i + 1].ch, p.kinsoku);
            if p.hanging_punct
                && mark
                && i > start
                // the line's content is then everything before the mark (spaces included)
                && x <= p.measure
                && next_ok
            {
                let why = if i + 1 == seg.len() {
                    None
                } else {
                    Some(LineEnd::Soft)
                };
                return Ok((i + 1, s.advance, why));
            }
            if let Some(b) = last_break {
                return Ok((b, 0, Some(LineEnd::Soft)));
            }
            if i == start {
                return Err(LayoutError::MeasureTooSmall {
                    needed: s.advance,
                    src: s.src,
                });
            }
            return Ok((i, 0, Some(LineEnd::Forced)));
        }
        x += s.advance + s.kern;
    }
    Ok((seg.len(), 0, None))
}

#[allow(clippy::too_many_arguments)]
fn push_line(
    lines: &mut Vec<Line>,
    glyphs: &[PlacedGlyph],
    first: usize,
    src: Range<u32>,
    width: i32,
    hang: i32,
    end: LineEnd,
    p: &LayoutParams,
) {
    // glyph and line counts stay far below 2^31 for any text under 4 GiB of glyphs we lay out
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let index = lines.len() as i32;
    #[allow(clippy::cast_possible_truncation)]
    let range = first as u32..glyphs.len() as u32;
    lines.push(Line {
        glyphs: range,
        src,
        width,
        hang,
        overflow: (width - p.measure).max(0),
        baseline: index * p.line_height + p.ascent,
        end,
    });
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

//! Reads TrueType / OpenType (sfnt) font files: the table directory and font collections (`ttcf`), `cmap`
//! (formats 0, 4, 6, 10, 12, 13 and variation sequences, 14), `glyf` / `loca` (quadratic outlines, composites),
//! `CFF ` / `CFF2` (Type 2 charstrings, CFF2 at its default instance) and horizontal metrics (`hhea` / `hmtx`).
//!
//! The input is untrusted: every count and offset is checked against the bytes left before anything is read
//! (sizes in u64, so nothing wraps), nesting and work are bounded, and every failure is an [`Error`]. Nothing
//! panics, in debug or in release builds.
//!
//! ```
//! # fn draw(bytes: &[u8]) -> Result<(), alice_sfnt::Error> {
//! let font = alice_sfnt::Font::parse(bytes, 0)?;
//! if let Some(gid) = font.glyph_index('A' as u32)? {
//!     let mut sink = alice_sfnt::Contours::default();
//!     font.outline(gid, &mut sink)?;
//!     let _ = (font.units_per_em(), font.advance(gid)?, sink.contours.len());
//! }
//! # Ok(()) }
//! ```

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod cff;
mod cmap;
mod error;
mod glyf;
mod reader;

use alloc::vec::Vec;

pub use cff::{MAX_OPS, MAX_SUBR_DEPTH};
pub use cmap::{Variation, MAX_CMAP_SUBTABLES};
pub use error::{Error, ErrorKind, Result};
pub use glyf::{MAX_DEPTH as MAX_COMPOSITE_DEPTH, MAX_GLYF_WORK};

use error::{malformed, truncated, unsupported};
use reader::{slice, Reader};

/// Receives an outline: contours of lines and quadratic (TrueType) or cubic (CFF) curves, in font units, y up.
pub trait OutlineSink {
    /// Starts a contour.
    fn move_to(&mut self, x: f32, y: f32);
    /// A straight segment to (x, y).
    fn line_to(&mut self, x: f32, y: f32);
    /// A quadratic segment with control point (x1, y1).
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32);
    /// A cubic segment with control points (x1, y1) and (x2, y2).
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32);
    /// Ends the contour (back to its start).
    fn close(&mut self);
}

/// One outline segment, as [`Contours`] records it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment {
    /// The contour starts here.
    Move(f32, f32),
    /// A line to the point.
    Line(f32, f32),
    /// A quadratic curve: control point, end point.
    Quad(f32, f32, f32, f32),
    /// A cubic curve: two control points, end point.
    Cubic(f32, f32, f32, f32, f32, f32),
}

/// An [`OutlineSink`] that keeps the segments, one `Vec` per contour.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Contours {
    /// The contours, each starting with a [`Segment::Move`].
    pub contours: Vec<Vec<Segment>>,
}

impl OutlineSink for Contours {
    fn move_to(&mut self, x: f32, y: f32) {
        self.contours.push(alloc::vec![Segment::Move(x, y)]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        if let Some(c) = self.contours.last_mut() {
            c.push(Segment::Line(x, y));
        }
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        if let Some(c) = self.contours.last_mut() {
            c.push(Segment::Quad(x1, y1, x, y));
        }
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        if let Some(c) = self.contours.last_mut() {
            c.push(Segment::Cubic(x1, y1, x2, y2, x, y));
        }
    }
    fn close(&mut self) {}
}

const TTCF: u32 = 0x7474_6366;
const SFNT_VERSIONS: [u32; 3] = [0x0001_0000, 0x7472_7565, 0x4F54_544F];

/// The number of fonts in `data`: the collection's count for a `ttcf`, 1 for a single font.
pub fn fonts_in(data: &[u8]) -> Result<u32> {
    let mut r = Reader::new(data, "font header");
    if r.u32()? != TTCF {
        return Ok(1);
    }
    r.skip(4)?;
    let n = r.u32()?;
    r.array(u64::from(n), 4)?;
    Ok(n)
}

#[derive(Clone, Copy)]
enum Outlines<'a> {
    None,
    Glyf(glyf::Glyf<'a>),
    Cff(cff::Cff<'a>),
}

/// A parsed font face.
#[derive(Clone)]
pub struct Font<'a> {
    units_per_em: u16,
    num_glyphs: u16,
    hmtx: Option<(&'a [u8], u16)>,
    cmap: Option<cmap::Cmap<'a>>,
    outlines: Outlines<'a>,
}

fn table<'a>(data: &'a [u8], records: &'a [u8], tag: &[u8; 4]) -> Result<Option<&'a [u8]>> {
    for rec in records.chunks_exact(16) {
        if &rec[..4] == tag {
            let mut r = Reader::new(&rec[8..], "table record");
            let (offset, len) = (r.u32()?, r.u32()?);
            return slice(data, u64::from(offset), u64::from(len), "table").map(Some);
        }
    }
    Ok(None)
}

fn required<'a>(t: Option<&'a [u8]>, context: &'static str) -> Result<&'a [u8]> {
    t.ok_or(Error::new(ErrorKind::MissingTable, context))
}

impl<'a> Font<'a> {
    /// Opens face `index` of `data` (0 for a single font). The directory, `head`, `maxp`, `hhea` / `hmtx`,
    /// `cmap` and the outline table's structure are checked here; glyphs are read when asked for.
    pub fn parse(data: &'a [u8], index: u32) -> Result<Self> {
        let mut r = Reader::new(data, "font header");
        let tag = r.u32()?;
        let start = if tag == TTCF {
            r.skip(4)?;
            let n = r.u32()?;
            let offsets = r.array(u64::from(n), 4)?;
            if index >= n {
                return Err(malformed("font index past the collection"));
            }
            let i = index as usize * 4;
            u64::from(u32::from_be_bytes([
                offsets[i],
                offsets[i + 1],
                offsets[i + 2],
                offsets[i + 3],
            ]))
        } else {
            if index != 0 {
                return Err(malformed("font index other than 0 for a single font"));
            }
            0
        };
        let mut r = Reader::at(data, start, "offset table")?;
        if !SFNT_VERSIONS.contains(&r.u32()?) {
            return Err(unsupported("sfnt version"));
        }
        let tables = r.u16()?;
        r.skip(6)?;
        let records = r.array(u64::from(tables), 16)?;

        let head = required(table(data, records, b"head")?, "head")?;
        let mut h = Reader::new(head, "head");
        h.skip(18)?;
        let units_per_em = h.u16()?;
        if !(16..=16384).contains(&units_per_em) {
            return Err(malformed("head unitsPerEm"));
        }
        h.skip(30)?;
        let loc_format = h.i16()?;

        let maxp = required(table(data, records, b"maxp")?, "maxp")?;
        let mut m = Reader::new(maxp, "maxp");
        m.skip(4)?;
        let num_glyphs = m.u16()?;
        if num_glyphs == 0 {
            return Err(malformed("maxp numGlyphs"));
        }

        let hhea = required(table(data, records, b"hhea")?, "hhea")?;
        let hmtx = match table(data, records, b"hmtx")? {
            Some(t) => {
                let mut hr = Reader::new(hhea, "hhea");
                hr.skip(34)?;
                let metrics = hr.u16()?;
                if metrics == 0 || (t.len() as u64) < u64::from(metrics) * 4 {
                    return Err(truncated("hmtx"));
                }
                Some((t, metrics))
            }
            None => None,
        };

        let cmap = match table(data, records, b"cmap")? {
            Some(t) => Some(cmap::Cmap::parse(t)?),
            None => None,
        };

        let outlines = if let Some(glyf) = table(data, records, b"glyf")? {
            let loca = required(table(data, records, b"loca")?, "loca")?;
            let long = match loc_format {
                0 => false,
                1 => true,
                _ => return Err(malformed("head indexToLocFormat")),
            };
            Outlines::Glyf(glyf::Glyf::new(loca, long, glyf, num_glyphs)?)
        } else if let Some(cff2) = table(data, records, b"CFF2")? {
            Outlines::Cff(cff::Cff::parse(cff2, true, num_glyphs)?)
        } else if let Some(cff) = table(data, records, b"CFF ")? {
            Outlines::Cff(cff::Cff::parse(cff, false, num_glyphs)?)
        } else {
            Outlines::None
        };

        Ok(Font {
            units_per_em,
            num_glyphs,
            hmtx,
            cmap,
            outlines,
        })
    }

    /// Font units per em (16..=16384).
    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    /// The number of glyphs (`maxp`), at least 1.
    pub fn num_glyphs(&self) -> u16 {
        self.num_glyphs
    }

    /// The glyph a code point maps to, `None` when no Unicode subtable maps it (or the font has no `cmap`).
    pub fn glyph_index(&self, code_point: u32) -> Result<Option<u16>> {
        match &self.cmap {
            Some(c) => c.glyph(code_point),
            None => Ok(None),
        }
    }

    /// The result of a variation sequence (`cmap` format 14), `None` when the font does not list it.
    pub fn glyph_variation(&self, code_point: u32, selector: u32) -> Result<Option<Variation>> {
        match &self.cmap {
            Some(c) => c.variation(code_point, selector),
            None => Ok(None),
        }
    }

    /// The horizontal advance in font units (`hmtx`; glyphs past the last metric use the last advance).
    pub fn advance(&self, gid: u16) -> Result<u16> {
        if gid >= self.num_glyphs {
            return Err(Error::new(ErrorKind::BadGlyph, "hmtx"));
        }
        let (t, n) = self
            .hmtx
            .ok_or(Error::new(ErrorKind::MissingTable, "hmtx"))?;
        let i = usize::from(gid.min(n - 1)) * 4;
        Reader::new(&t[i..i + 2], "hmtx").u16()
    }

    /// Draws glyph `gid` into `sink`. `Ok(false)` when the glyph has no outline (a space, or a font without
    /// `glyf` / `CFF`); on `Err` the sink may hold a partial outline, which the caller should drop.
    pub fn outline(&self, gid: u16, sink: &mut dyn OutlineSink) -> Result<bool> {
        if gid >= self.num_glyphs {
            return Err(Error::new(ErrorKind::BadGlyph, "outline"));
        }
        match &self.outlines {
            Outlines::None => Ok(false),
            Outlines::Glyf(g) => g.outline(gid, sink),
            Outlines::Cff(c) => c.outline(gid, sink),
        }
    }
}

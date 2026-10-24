//! `glyf` / `loca`: TrueType outlines (quadratic), simple and composite.
//!
//! Point counts, flag repeats and coordinate arrays are checked against the glyph's bytes before the points are
//! read; composite glyphs nest at most [`MAX_DEPTH`] deep.

use crate::error::{limit, malformed, truncated, Error, ErrorKind, Result};
use crate::reader::{slice, Reader};
use crate::OutlineSink;
use alloc::vec::Vec;

/// How deep composite glyphs may nest (the spec sets no bound; this is the same as ttf-parser's).
pub const MAX_DEPTH: u8 = 32;

#[derive(Clone, Copy)]
pub(crate) struct Glyf<'a> {
    loca: &'a [u8],
    long: bool,
    glyf: &'a [u8],
    num_glyphs: u16,
}

#[derive(Clone, Copy)]
struct Transform {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Transform {
    const IDENTITY: Transform = Transform {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    fn is_identity(&self) -> bool {
        self.a == 1.0
            && self.b == 0.0
            && self.c == 0.0
            && self.d == 1.0
            && self.e == 0.0
            && self.f == 0.0
    }

    /// `self` applied after `inner` (the same composition as ttf-parser's `Transform::combine`)
    fn then(self, inner: Transform) -> Transform {
        let (t1, t2) = (self, inner);
        Transform {
            a: t1.a * t2.a + t1.c * t2.b,
            b: t1.b * t2.a + t1.d * t2.b,
            c: t1.a * t2.c + t1.c * t2.d,
            d: t1.b * t2.c + t1.d * t2.d,
            e: t1.a * t2.e + t1.c * t2.f + t1.e,
            f: t1.b * t2.e + t1.d * t2.f + t1.f,
        }
    }

    fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        if self.is_identity() {
            return (x, y);
        }
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }
}

/// Turns TrueType points (on / off curve) into move / line / quad / close.
struct Builder<'s> {
    sink: &'s mut dyn OutlineSink,
    ts: Transform,
    first_on: Option<(f32, f32)>,
    first_off: Option<(f32, f32)>,
    last_off: Option<(f32, f32)>,
    drew: bool,
}

fn mid(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    // the same as ttf-parser's lerp(a, b, 0.5)
    (a.0 + 0.5 * (b.0 - a.0), a.1 + 0.5 * (b.1 - a.1))
}

impl Builder<'_> {
    fn move_to(&mut self, p: (f32, f32)) {
        let (x, y) = self.ts.apply(p.0, p.1);
        self.drew = true;
        self.sink.move_to(x, y);
    }

    fn line_to(&mut self, p: (f32, f32)) {
        let (x, y) = self.ts.apply(p.0, p.1);
        self.sink.line_to(x, y);
    }

    fn quad_to(&mut self, c: (f32, f32), p: (f32, f32)) {
        let (cx, cy) = self.ts.apply(c.0, c.1);
        let (x, y) = self.ts.apply(p.0, p.1);
        self.sink.quad_to(cx, cy, x, y);
    }

    fn point(&mut self, p: (f32, f32), on: bool, last: bool) {
        if self.first_on.is_none() {
            if on {
                self.first_on = Some(p);
                self.move_to(p);
            } else if let Some(off) = self.first_off {
                let m = mid(off, p);
                self.first_on = Some(m);
                self.last_off = Some(p);
                self.move_to(m);
            } else {
                self.first_off = Some(p);
            }
        } else {
            match (self.last_off, on) {
                (Some(off), true) => {
                    self.last_off = None;
                    self.quad_to(off, p);
                }
                (Some(off), false) => {
                    self.last_off = Some(p);
                    self.quad_to(off, mid(off, p));
                }
                (None, true) => self.line_to(p),
                (None, false) => self.last_off = Some(p),
            }
        }
        if last {
            self.finish();
        }
    }

    fn finish(&mut self) {
        if let (Some(f), Some(l)) = (self.first_off, self.last_off) {
            self.last_off = None;
            self.quad_to(l, mid(l, f));
        }
        if let (Some(p), Some(f)) = (self.first_on, self.first_off) {
            self.quad_to(f, p);
        } else if let (Some(p), Some(l)) = (self.first_on, self.last_off) {
            self.quad_to(l, p);
        } else if let Some(p) = self.first_on {
            self.line_to(p);
        }
        // a contour of off-curve points only that never got a start point draws nothing
        if self.first_on.is_some() {
            self.sink.close();
        }
        self.first_on = None;
        self.first_off = None;
        self.last_off = None;
    }
}

impl<'a> Glyf<'a> {
    pub(crate) fn new(loca: &'a [u8], long: bool, glyf: &'a [u8], num_glyphs: u16) -> Result<Self> {
        let size = if long { 4 } else { 2 };
        // loca has numGlyphs + 1 offsets
        if (loca.len() as u64) < (u64::from(num_glyphs) + 1) * size {
            return Err(truncated("loca"));
        }
        Ok(Glyf {
            loca,
            long,
            glyf,
            num_glyphs,
        })
    }

    fn range(&self, gid: u16) -> Result<&'a [u8]> {
        if gid >= self.num_glyphs {
            return Err(Error::new(ErrorKind::BadGlyph, "glyf"));
        }
        let i = usize::from(gid);
        let (start, end) = if self.long {
            let mut r = Reader::new(&self.loca[i * 4..i * 4 + 8], "loca");
            (u64::from(r.u32()?), u64::from(r.u32()?))
        } else {
            let mut r = Reader::new(&self.loca[i * 2..i * 2 + 4], "loca");
            (u64::from(r.u16()?) * 2, u64::from(r.u16()?) * 2)
        };
        if end < start {
            return Err(malformed("loca offsets out of order"));
        }
        slice(self.glyf, start, end - start, "glyf glyph")
    }

    /// Draws the glyph into `sink`; `Ok(false)` when it has no outline (an empty glyph such as a space).
    pub(crate) fn outline(&self, gid: u16, sink: &mut dyn OutlineSink) -> Result<bool> {
        let mut b = Builder {
            sink,
            ts: Transform::IDENTITY,
            first_on: None,
            first_off: None,
            last_off: None,
            drew: false,
        };
        self.draw(gid, Transform::IDENTITY, 0, &mut b)?;
        Ok(b.drew)
    }

    fn draw(&self, gid: u16, ts: Transform, depth: u8, b: &mut Builder<'_>) -> Result<()> {
        if depth >= MAX_DEPTH {
            return Err(limit("glyf composite depth"));
        }
        let data = self.range(gid)?;
        if data.is_empty() {
            return Ok(());
        }
        let mut r = Reader::new(data, "glyf header");
        let contours = r.i16()?;
        r.skip(8)?; // bounding box: the outline itself is the source of truth
        let body = &data[10..];
        if contours > 0 {
            b.ts = ts;
            simple(body, contours as u16, b)
        } else if contours < 0 {
            composite(self, body, ts, depth, b)
        } else {
            Ok(())
        }
    }
}

fn simple(body: &[u8], contours: u16, b: &mut Builder<'_>) -> Result<()> {
    let mut r = Reader::new(body, "glyf simple glyph");
    let ends = r.array(u64::from(contours), 2)?;
    let mut end_points = Vec::with_capacity(usize::from(contours));
    let mut prev: Option<u16> = None;
    for e in ends.chunks_exact(2) {
        let e = u16::from_be_bytes([e[0], e[1]]);
        if prev.is_some_and(|p| e < p) {
            return Err(malformed("glyf endPtsOfContours not increasing"));
        }
        prev = Some(e);
        end_points.push(e);
    }
    let total = u32::from(*end_points.last().ok_or(malformed("glyf simple glyph"))?) + 1;
    // a glyph of a single point has no outline (the same rule as ttf-parser)
    if total == 1 {
        return Ok(());
    }
    let ins = r.u16()?;
    r.skip(u64::from(ins))?;
    // flags, expanded; repeats may not run past the point count
    let mut flags = Vec::with_capacity(total as usize);
    let (mut x_len, mut y_len) = (0u64, 0u64);
    while (flags.len() as u32) < total {
        let f = r.u8()?;
        let n = if f & 0x08 != 0 {
            u32::from(r.u8()?) + 1
        } else {
            1
        };
        if n > total - flags.len() as u32 {
            return Err(malformed("glyf flag repeat past the point count"));
        }
        let xs = if f & 0x02 != 0 {
            1
        } else if f & 0x10 != 0 {
            0
        } else {
            2
        };
        let ys = if f & 0x04 != 0 {
            1
        } else if f & 0x20 != 0 {
            0
        } else {
            2
        };
        x_len += xs * u64::from(n);
        y_len += ys * u64::from(n);
        for _ in 0..n {
            flags.push(f);
        }
    }
    let mut xr = Reader::new(r.bytes(x_len)?, "glyf x coordinates");
    let mut yr = Reader::new(r.bytes(y_len)?, "glyf y coordinates");
    let (mut x, mut y) = (0i16, 0i16);
    let mut contour = 0usize;
    for (i, &f) in flags.iter().enumerate() {
        x = x.wrapping_add(delta(&mut xr, f, 0x02, 0x10)?);
        y = y.wrapping_add(delta(&mut yr, f, 0x04, 0x20)?);
        let last = i as u32 == u32::from(end_points[contour]);
        b.point((f32::from(x), f32::from(y)), f & 0x01 != 0, last);
        if last {
            contour += 1;
            // repeated end points describe empty contours: skip past them
            while contour < end_points.len() && u32::from(end_points[contour]) == i as u32 {
                contour += 1;
            }
            if contour == end_points.len() {
                break;
            }
        }
    }
    Ok(())
}

fn delta(r: &mut Reader<'_>, f: u8, short: u8, same_or_positive: u8) -> Result<i16> {
    if f & short != 0 {
        let v = i16::from(r.u8()?);
        Ok(if f & same_or_positive != 0 { v } else { -v })
    } else if f & same_or_positive != 0 {
        Ok(0)
    } else {
        r.i16()
    }
}

fn f2dot14(r: &mut Reader<'_>) -> Result<f32> {
    Ok(f32::from(r.i16()?) / 16384.0)
}

fn composite(
    g: &Glyf<'_>,
    body: &[u8],
    ts: Transform,
    depth: u8,
    b: &mut Builder<'_>,
) -> Result<()> {
    let mut r = Reader::new(body, "glyf composite glyph");
    loop {
        let flags = r.u16()?;
        let child = r.u16()?;
        let mut t = Transform::IDENTITY;
        if flags & 0x0002 != 0 {
            if flags & 0x0001 != 0 {
                t.e = f32::from(r.i16()?);
                t.f = f32::from(r.i16()?);
            } else {
                t.e = f32::from(r.i8()?);
                t.f = f32::from(r.i8()?);
            }
        } else {
            // point-matching arguments: read and not applied (as ttf-parser does)
            r.skip(if flags & 0x0001 != 0 { 4 } else { 2 })?;
        }
        if flags & 0x0080 != 0 {
            t.a = f2dot14(&mut r)?;
            t.b = f2dot14(&mut r)?;
            t.c = f2dot14(&mut r)?;
            t.d = f2dot14(&mut r)?;
        } else if flags & 0x0040 != 0 {
            t.a = f2dot14(&mut r)?;
            t.d = f2dot14(&mut r)?;
        } else if flags & 0x0008 != 0 {
            t.a = f2dot14(&mut r)?;
            t.d = t.a;
        }
        // a component that names a glyph outside the font is an error (ttf-parser skips it)
        g.draw(child, ts.then(t), depth + 1, b)?;
        if flags & 0x0020 == 0 {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use core::fmt::Write;

    #[derive(Default)]
    struct Log(String);

    impl OutlineSink for Log {
        fn move_to(&mut self, x: f32, y: f32) {
            let _ = write!(self.0, "M{x},{y} ");
        }
        fn line_to(&mut self, x: f32, y: f32) {
            let _ = write!(self.0, "L{x},{y} ");
        }
        fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
            let _ = write!(self.0, "Q{x1},{y1},{x},{y} ");
        }
        fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
            self.0.push_str("C ");
        }
        fn close(&mut self) {
            self.0.push_str("Z ");
        }
    }

    fn glyph_simple(points: &[(i16, i16, bool)], ends: &[u16]) -> Vec<u8> {
        let mut g = Vec::new();
        g.extend_from_slice(&(ends.len() as i16).to_be_bytes());
        g.extend_from_slice(&[0; 8]);
        for e in ends {
            g.extend_from_slice(&e.to_be_bytes());
        }
        g.extend_from_slice(&0u16.to_be_bytes());
        for &(_, _, on) in points {
            g.push(u8::from(on));
        }
        let (mut px, mut py) = (0i16, 0i16);
        for &(x, _, _) in points {
            g.extend_from_slice(&(x - px).to_be_bytes());
            px = x;
        }
        for &(_, y, _) in points {
            g.extend_from_slice(&(y - py).to_be_bytes());
            py = y;
        }
        g
    }

    fn font_of(glyphs: &[Vec<u8>]) -> (Vec<u8>, Vec<u8>) {
        let mut loca = Vec::new();
        let mut glyf = Vec::new();
        for g in glyphs {
            loca.extend_from_slice(&(glyf.len() as u32).to_be_bytes());
            glyf.extend_from_slice(g);
        }
        loca.extend_from_slice(&(glyf.len() as u32).to_be_bytes());
        (loca, glyf)
    }

    #[test]
    fn a_triangle_draws_lines_and_closes() {
        let (loca, glyf) = font_of(&[glyph_simple(
            &[(0, 0, true), (100, 0, true), (50, 80, true)],
            &[2],
        )]);
        let g = Glyf::new(&loca, true, &glyf, 1).unwrap();
        let mut log = Log::default();
        assert!(g.outline(0, &mut log).unwrap());
        assert_eq!(log.0, "M0,0 L100,0 L50,80 L0,0 Z ");
    }

    #[test]
    fn off_curve_points_make_implied_on_curve_midpoints() {
        // on, off, off, on: the two off points imply an on point between them
        let pts = [
            (0, 0, true),
            (0, 100, false),
            (100, 100, false),
            (100, 0, true),
        ];
        let (loca, glyf) = font_of(&[glyph_simple(&pts, &[3])]);
        let g = Glyf::new(&loca, true, &glyf, 1).unwrap();
        let mut log = Log::default();
        g.outline(0, &mut log).unwrap();
        assert_eq!(log.0, "M0,0 Q0,100,50,100 Q100,100,100,0 L0,0 Z ");
    }

    #[test]
    fn a_flag_repeat_past_the_points_is_an_error() {
        let mut g = glyph_simple(&[(0, 0, true), (1, 1, true), (2, 0, true)], &[2]);
        // flags start after 10 + 2 + 2 bytes: make the first flag repeat 200 times
        g[14] = 0x01 | 0x08;
        g.insert(15, 200);
        let (loca, glyf) = font_of(&[g]);
        let e = Glyf::new(&loca, true, &glyf, 1)
            .unwrap()
            .outline(0, &mut Log::default())
            .unwrap_err();
        assert_eq!(e.kind, ErrorKind::Malformed);
    }

    /// glyph k is a composite of glyph k + 1 for k < n, glyph n a triangle: drawing glyph 0 nests n deep
    fn chain(n: u16) -> (Vec<u8>, Vec<u8>) {
        let mut glyphs = Vec::new();
        for k in 0..n {
            let mut c = Vec::new();
            c.extend_from_slice(&(-1i16).to_be_bytes());
            c.extend_from_slice(&[0; 8]);
            c.extend_from_slice(&0x0002u16.to_be_bytes());
            c.extend_from_slice(&(k + 1).to_be_bytes());
            c.extend_from_slice(&[0, 0]);
            glyphs.push(c);
        }
        glyphs.push(glyph_simple(
            &[(0, 0, true), (10, 0, true), (5, 8, true)],
            &[2],
        ));
        font_of(&glyphs)
    }

    #[test]
    fn composites_nest_up_to_the_depth_limit() {
        let depth = u16::from(MAX_DEPTH);
        let (loca, glyf) = chain(depth - 1);
        let g = Glyf::new(&loca, true, &glyf, depth).unwrap();
        assert!(g.outline(0, &mut Log::default()).unwrap());
        let (loca, glyf) = chain(depth);
        let g = Glyf::new(&loca, true, &glyf, depth + 1).unwrap();
        assert_eq!(
            g.outline(0, &mut Log::default()).unwrap_err().kind,
            ErrorKind::Limit
        );
    }

    #[test]
    fn a_composite_that_names_itself_hits_the_depth_limit() {
        let mut c = Vec::new();
        c.extend_from_slice(&(-1i16).to_be_bytes());
        c.extend_from_slice(&[0; 8]);
        c.extend_from_slice(&0x0002u16.to_be_bytes()); // xy values, byte args, no more components
        c.extend_from_slice(&0u16.to_be_bytes());
        c.extend_from_slice(&[0, 0]);
        let (loca, glyf) = font_of(&[c]);
        let e = Glyf::new(&loca, true, &glyf, 1)
            .unwrap()
            .outline(0, &mut Log::default())
            .unwrap_err();
        assert_eq!(e.kind, ErrorKind::Limit);
    }

    #[test]
    fn a_composite_moves_and_scales_its_component() {
        let tri = glyph_simple(&[(0, 0, true), (100, 0, true), (50, 80, true)], &[2]);
        let mut c = Vec::new();
        c.extend_from_slice(&(-1i16).to_be_bytes());
        c.extend_from_slice(&[0; 8]);
        c.extend_from_slice(&(0x0001u16 | 0x0002 | 0x0008).to_be_bytes()); // word args, xy, one scale
        c.extend_from_slice(&0u16.to_be_bytes());
        c.extend_from_slice(&10i16.to_be_bytes());
        c.extend_from_slice(&20i16.to_be_bytes());
        c.extend_from_slice(&0x2000i16.to_be_bytes()); // 0.5
        let (loca, glyf) = font_of(&[tri, c]);
        let mut log = Log::default();
        Glyf::new(&loca, true, &glyf, 2)
            .unwrap()
            .outline(1, &mut log)
            .unwrap();
        assert_eq!(log.0, "M10,20 L60,20 L35,60 L10,20 Z ");
    }

    #[test]
    fn loca_shorter_than_the_glyph_count_is_an_error() {
        assert_eq!(
            Glyf::new(&[0; 6], false, &[], 3).err().unwrap().kind,
            ErrorKind::Truncated
        );
        assert_eq!(
            Glyf::new(&[0; 8], false, &[], 3)
                .unwrap()
                .outline(3, &mut Log::default())
                .unwrap_err()
                .kind,
            ErrorKind::BadGlyph
        );
    }
}

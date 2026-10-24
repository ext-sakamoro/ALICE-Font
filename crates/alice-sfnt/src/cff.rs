//! `CFF ` and `CFF2`: Type 2 charstring outlines (cubic).
//!
//! INDEX counts and offsets are checked against the table before an entry is taken, the operand stack is bounded
//! (48 for CFF, 513 for CFF2), subroutines nest at most [`MAX_SUBR_DEPTH`] deep and a glyph runs at most
//! [`MAX_OPS`] operators. CFF2 is read at the default instance (blend keeps the default values).

use crate::error::{limit, malformed, truncated, unsupported, Result};
use crate::reader::{slice, tail, Reader};
use crate::OutlineSink;

/// How deep subroutine calls may nest (the Type 2 limit).
pub const MAX_SUBR_DEPTH: u8 = 10;
/// How many operators and operands one glyph may run, counting subroutine bodies each time they are called.
pub const MAX_OPS: u32 = 1 << 20;

/// An INDEX: `count` entries, offsets of `off_size` bytes, data after the offsets.
#[derive(Clone, Copy)]
struct Index<'a> {
    count: u32,
    off_size: u8,
    offsets: &'a [u8],
    data: &'a [u8],
}

impl<'a> Index<'a> {
    /// reads an INDEX at the start of `r` (count is u16 in CFF, u32 in CFF2) and moves `r` past it
    fn read(r: &mut Reader<'a>, cff2: bool) -> Result<Self> {
        let count = if cff2 { r.u32()? } else { u32::from(r.u16()?) };
        if count == 0 {
            return Ok(Index {
                count,
                off_size: 1,
                offsets: &[],
                data: &[],
            });
        }
        let off_size = r.u8()?;
        if !(1..=4).contains(&off_size) {
            return Err(malformed("CFF INDEX offSize"));
        }
        let offsets = r.array(u64::from(count) + 1, u64::from(off_size))?;
        let last = Reader::new(&offsets[offsets.len() - off_size as usize..], "CFF INDEX")
            .uint(off_size)?;
        if last == 0 {
            return Err(malformed("CFF INDEX offsets"));
        }
        let data = r.bytes(u64::from(last) - 1)?;
        Ok(Index {
            count,
            off_size,
            offsets,
            data,
        })
    }

    fn get(&self, i: u32) -> Result<&'a [u8]> {
        if i >= self.count {
            return Err(malformed("CFF INDEX entry"));
        }
        let size = u64::from(self.off_size);
        let at = |k: u32| -> Result<u64> {
            let b = slice(self.offsets, u64::from(k) * size, size, "CFF INDEX offset")?;
            Ok(u64::from(
                Reader::new(b, "CFF INDEX offset").uint(self.off_size)?,
            ))
        };
        let (s, e) = (at(i)?, at(i + 1)?);
        if s == 0 || e < s {
            return Err(malformed("CFF INDEX offsets"));
        }
        slice(self.data, s - 1, e - s, "CFF INDEX entry")
    }
}

/// DICT operands and the operator that ends them.
struct DictEntry {
    op: u16,
    n: usize,
    args: [f64; 513],
}

fn dict_entries(data: &[u8], mut f: impl FnMut(&DictEntry) -> Result<()>) -> Result<()> {
    let mut r = Reader::new(data, "CFF DICT");
    // CFF allows 48 operands, CFF2 513
    let mut e = DictEntry {
        op: 0,
        n: 0,
        args: [0.0; 513],
    };
    while !r.at_end() {
        let b0 = r.u8()?;
        let value = match b0 {
            // 0..=21 in CFF; CFF2 adds 22 (vsindex), 23 (blend) and 24 (vstore). blend's operands only feed keys
            // this crate does not read (hint values), so it ends its operands like any other operator
            0..=27 => {
                e.op = if b0 == 12 {
                    1200 + u16::from(r.u8()?)
                } else {
                    u16::from(b0)
                };
                f(&e)?;
                e.n = 0;
                continue;
            }
            28 => f64::from(r.i16()?),
            29 => f64::from(r.i32()?),
            30 => {
                // a real number in nibbles; its value is only used for keys this crate ignores
                loop {
                    let b = r.u8()?;
                    if b & 0x0F == 0x0F || b >> 4 == 0x0F {
                        break;
                    }
                }
                0.0
            }
            32..=246 => f64::from(i32::from(b0) - 139),
            247..=250 => f64::from((i32::from(b0) - 247) * 256 + i32::from(r.u8()?) + 108),
            251..=254 => f64::from(-(i32::from(b0) - 251) * 256 - i32::from(r.u8()?) - 108),
            _ => return Err(malformed("CFF DICT operand")),
        };
        if e.n >= e.args.len() {
            return Err(limit("CFF DICT operand stack"));
        }
        e.args[e.n] = value;
        e.n += 1;
    }
    Ok(())
}

fn as_offset(v: f64, context: &'static str) -> Result<u64> {
    if !(0.0..=f64::from(u32::MAX)).contains(&v) || v != (v as u64) as f64 {
        return Err(malformed(context));
    }
    Ok(v as u64)
}

/// Where one glyph's local subroutines come from.
#[derive(Clone, Copy)]
enum Locals<'a> {
    One(Option<Index<'a>>),
    /// CID-keyed (CFF) or CFF2: FDSelect picks a Font DICT per glyph
    PerFd {
        fd_array: Index<'a>,
        fd_select: &'a [u8],
        table: &'a [u8],
    },
}

#[derive(Clone, Copy)]
pub(crate) struct Cff<'a> {
    cff2: bool,
    charstrings: Index<'a>,
    globals: Index<'a>,
    locals: Locals<'a>,
    /// CFF2: region counts of the variation store's item data, indexed by vsindex
    vstore: Option<&'a [u8]>,
    num_glyphs: u16,
}

fn private_subrs<'a>(
    table: &'a [u8],
    size: u64,
    offset: u64,
    cff2: bool,
) -> Result<Option<Index<'a>>> {
    let private = slice(table, offset, size, "CFF Private DICT")?;
    let mut subrs = None;
    dict_entries(private, |e| {
        if e.op == 19 && e.n >= 1 {
            subrs = Some(as_offset(e.args[e.n - 1], "CFF Subrs offset")?);
        }
        Ok(())
    })?;
    match subrs {
        Some(o) => {
            let at = offset.checked_add(o).ok_or(truncated("CFF Subrs"))?;
            Ok(Some(Index::read(
                &mut Reader::at(table, at, "CFF Subrs")?,
                cff2,
            )?))
        }
        None => Ok(None),
    }
}

fn font_dict_subrs<'a>(table: &'a [u8], dict: &'a [u8], cff2: bool) -> Result<Option<Index<'a>>> {
    let mut private = None;
    dict_entries(dict, |e| {
        if e.op == 18 && e.n >= 2 {
            private = Some((
                as_offset(e.args[0], "CFF Private size")?,
                as_offset(e.args[1], "CFF Private offset")?,
            ));
        }
        Ok(())
    })?;
    match private {
        Some((size, offset)) => private_subrs(table, size, offset, cff2),
        None => Ok(None),
    }
}

impl<'a> Cff<'a> {
    pub(crate) fn parse(table: &'a [u8], cff2: bool, num_glyphs: u16) -> Result<Self> {
        let mut r = Reader::new(table, "CFF header");
        let major = r.u8()?;
        let _minor = r.u8()?;
        let hdr_size = r.u8()?;
        if major != if cff2 { 2 } else { 1 } {
            return Err(unsupported("CFF major version"));
        }
        let (top, globals) = if cff2 {
            let top_len = r.u16()?;
            let top = slice(
                table,
                u64::from(hdr_size),
                u64::from(top_len),
                "CFF2 Top DICT",
            )?;
            let mut g = Reader::at(
                table,
                u64::from(hdr_size) + u64::from(top_len),
                "CFF2 Global Subr INDEX",
            )?;
            (top, Index::read(&mut g, true)?)
        } else {
            let mut ir = Reader::at(table, u64::from(hdr_size), "CFF Name INDEX")?;
            let names = Index::read(&mut ir, false)?;
            if names.count != 1 {
                return Err(unsupported("CFF with more than one font"));
            }
            let tops = Index::read(&mut ir, false)?;
            let top = tops.get(0)?;
            let _strings = Index::read(&mut ir, false)?;
            (top, Index::read(&mut ir, false)?)
        };
        let (
            mut charstrings,
            mut private,
            mut fd_array,
            mut fd_select,
            mut vstore,
            mut cid,
            mut cs_type,
        ) = (None, None, None, None, None, false, 2.0);
        dict_entries(top, |e| {
            let last = if e.n > 0 { e.args[e.n - 1] } else { -1.0 };
            match e.op {
                17 => charstrings = Some(as_offset(last, "CFF CharStrings offset")?),
                18 if e.n >= 2 => {
                    private = Some((
                        as_offset(e.args[0], "CFF Private size")?,
                        as_offset(e.args[1], "CFF Private offset")?,
                    ))
                }
                24 => vstore = Some(as_offset(last, "CFF2 vstore offset")?),
                1206 => cs_type = last,
                1230 => cid = true,
                1236 => fd_array = Some(as_offset(last, "CFF FDArray offset")?),
                1237 => fd_select = Some(as_offset(last, "CFF FDSelect offset")?),
                _ => {}
            }
            Ok(())
        })?;
        if cs_type != 2.0 {
            return Err(unsupported("CFF CharstringType other than 2"));
        }
        let cs_off = charstrings.ok_or(malformed("CFF Top DICT has no CharStrings"))?;
        let charstrings = Index::read(&mut Reader::at(table, cs_off, "CFF CharStrings")?, cff2)?;
        if charstrings.count < u32::from(num_glyphs) {
            return Err(malformed("CFF CharStrings fewer than the glyph count"));
        }
        let locals = if cff2 || cid {
            let fda = fd_array.ok_or(malformed("CFF FDArray"))?;
            let fd_array = Index::read(&mut Reader::at(table, fda, "CFF FDArray")?, cff2)?;
            let fd_select = match fd_select {
                Some(o) => tail(table, o, "CFF FDSelect")?,
                // CFF2 with one Font DICT may omit FDSelect
                None if cff2 && fd_array.count == 1 => &[],
                None => return Err(malformed("CFF FDSelect")),
            };
            Locals::PerFd {
                fd_array,
                fd_select,
                table,
            }
        } else {
            Locals::One(match private {
                Some((size, offset)) => private_subrs(table, size, offset, false)?,
                None => None,
            })
        };
        let vstore = match vstore {
            Some(o) if cff2 => Some(tail(table, o, "CFF2 vstore")?),
            _ => None,
        };
        Ok(Cff {
            cff2,
            charstrings,
            globals,
            locals,
            vstore,
            num_glyphs,
        })
    }

    fn fd_of(fd_select: &[u8], gid: u16, fds: u32) -> Result<u32> {
        if fd_select.is_empty() {
            return Ok(0);
        }
        let mut r = Reader::new(fd_select, "CFF FDSelect");
        let fd = match r.u8()? {
            0 => {
                r.skip(u64::from(gid))?;
                u32::from(r.u8()?)
            }
            3 => {
                let n = r.u16()?;
                let ranges = r.array(u64::from(n), 3)?;
                let sentinel = r.u16()?;
                let mut fd = None;
                for (i, rg) in ranges.chunks_exact(3).enumerate() {
                    let first = u16::from_be_bytes([rg[0], rg[1]]);
                    let next = if i + 1 < usize::from(n) {
                        u16::from_be_bytes([ranges[(i + 1) * 3], ranges[(i + 1) * 3 + 1]])
                    } else {
                        sentinel
                    };
                    if gid >= first && gid < next {
                        fd = Some(u32::from(rg[2]));
                        break;
                    }
                }
                fd.ok_or(malformed("CFF FDSelect has no range for the glyph"))?
            }
            4 => {
                let n = r.u32()?;
                let ranges = r.array(u64::from(n), 6)?;
                let sentinel = r.u32()?;
                let mut fd = None;
                for (i, rg) in ranges.chunks_exact(6).enumerate() {
                    let first = u32::from_be_bytes([rg[0], rg[1], rg[2], rg[3]]);
                    let next = if (i as u64) + 1 < u64::from(n) {
                        let b = &ranges[(i + 1) * 6..(i + 1) * 6 + 4];
                        u32::from_be_bytes([b[0], b[1], b[2], b[3]])
                    } else {
                        sentinel
                    };
                    if u32::from(gid) >= first && u32::from(gid) < next {
                        fd = Some(u32::from(u16::from_be_bytes([rg[4], rg[5]])));
                        break;
                    }
                }
                fd.ok_or(malformed("CFF FDSelect has no range for the glyph"))?
            }
            _ => return Err(unsupported("CFF FDSelect format")),
        };
        if fd >= fds {
            return Err(malformed("CFF FDSelect names a Font DICT past FDArray"));
        }
        Ok(fd)
    }

    fn region_count(&self, vsindex: u32) -> Result<u16> {
        let Some(vs) = self.vstore else {
            return Err(malformed("CFF2 blend without a variation store"));
        };
        // the CFF2 vstore starts with a u16 length, then an ItemVariationStore
        let store = tail(vs, 2, "CFF2 vstore")?;
        let mut r = Reader::new(store, "CFF2 ItemVariationStore");
        let _format = r.u16()?;
        let _regions = r.u32()?;
        let count = r.u16()?;
        let offsets = r.array(u64::from(count), 4)?;
        if vsindex >= u32::from(count) {
            return Err(malformed("CFF2 vsindex"));
        }
        let i = vsindex as usize * 4;
        let off = u32::from_be_bytes([offsets[i], offsets[i + 1], offsets[i + 2], offsets[i + 3]]);
        let mut d = Reader::at(store, u64::from(off), "CFF2 ItemVariationData")?;
        d.skip(4)?; // itemCount, wordDeltaCount
        d.u16()
    }

    pub(crate) fn outline(&self, gid: u16, sink: &mut dyn OutlineSink) -> Result<bool> {
        if gid >= self.num_glyphs {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::BadGlyph,
                "CFF CharStrings",
            ));
        }
        let locals = match self.locals {
            Locals::One(i) => i,
            Locals::PerFd {
                fd_array,
                fd_select,
                table,
            } => {
                let fd = Self::fd_of(fd_select, gid, fd_array.count)?;
                font_dict_subrs(table, fd_array.get(fd)?, self.cff2)?
            }
        };
        let cs = self.charstrings.get(u32::from(gid))?;
        let mut m = Machine {
            cff: self,
            locals,
            sink,
            stack: [0.0; 513],
            len: 0,
            max: if self.cff2 { 513 } else { 48 },
            x: 0.0,
            y: 0.0,
            open: false,
            drew: false,
            stems: 0,
            width_seen: self.cff2,
            ended: false,
            ops: 0,
            vsindex: 0,
        };
        m.run(cs, 0)?;
        if !self.cff2 && !m.ended {
            return Err(malformed("CFF charstring without endchar"));
        }
        if m.open {
            m.sink.close();
        }
        Ok(m.drew)
    }
}

fn abs(v: f32) -> f32 {
    if v < 0.0 {
        -v
    } else {
        v
    }
}

fn bias(count: u32) -> i64 {
    if count < 1240 {
        107
    } else if count < 33900 {
        1131
    } else {
        32768
    }
}

struct Machine<'c, 'a, 's> {
    cff: &'c Cff<'a>,
    locals: Option<Index<'a>>,
    sink: &'s mut dyn OutlineSink,
    stack: [f32; 513],
    len: usize,
    max: usize,
    x: f32,
    y: f32,
    open: bool,
    drew: bool,
    stems: u32,
    /// CFF: the optional width operand of the first stack-clearing operator has been taken
    width_seen: bool,
    ended: bool,
    ops: u32,
    vsindex: u32,
}

impl Machine<'_, '_, '_> {
    fn push(&mut self, v: f32) -> Result<()> {
        if self.len >= self.max {
            return Err(limit("CFF operand stack"));
        }
        self.stack[self.len] = v;
        self.len += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<f32> {
        if self.len == 0 {
            return Err(malformed("CFF operand stack underflow"));
        }
        self.len -= 1;
        Ok(self.stack[self.len])
    }

    /// drops the leading width operand of the first stack-clearing operator when the count says it is there
    fn take_width(&mut self, expected_odd: bool) {
        if !self.width_seen {
            self.width_seen = true;
            if (self.len % 2 == 1) == expected_odd && self.len > 0 {
                self.stack.copy_within(1..self.len, 0);
                self.len -= 1;
            }
        }
    }

    fn need_move(&self) -> Result<()> {
        if self.open {
            Ok(())
        } else {
            Err(malformed("CFF path operator before a moveto"))
        }
    }

    fn move_to(&mut self, dx: f32, dy: f32) {
        if self.open {
            self.sink.close();
        }
        self.x += dx;
        self.y += dy;
        self.sink.move_to(self.x, self.y);
        self.open = true;
        self.drew = true;
    }

    fn line(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
        self.sink.line_to(self.x, self.y);
    }

    fn curve(&mut self, d: [f32; 6]) {
        let x1 = self.x + d[0];
        let y1 = self.y + d[1];
        let x2 = x1 + d[2];
        let y2 = y1 + d[3];
        self.x = x2 + d[4];
        self.y = y2 + d[5];
        self.sink.curve_to(x1, y1, x2, y2, self.x, self.y);
    }

    fn arg(&self, i: usize) -> f32 {
        self.stack[i]
    }

    fn run(&mut self, cs: &[u8], depth: u8) -> Result<()> {
        if depth > MAX_SUBR_DEPTH {
            return Err(limit("CFF subroutine depth"));
        }
        let mut r = Reader::new(cs, "CFF charstring");
        while !r.at_end() {
            self.ops += 1;
            if self.ops > MAX_OPS {
                return Err(limit("CFF charstring work"));
            }
            let op = r.u8()?;
            match op {
                32..=246 => self.push(f32::from(i16::from(op) - 139))?,
                247..=250 => self.push(f32::from(
                    (i16::from(op) - 247) * 256 + i16::from(r.u8()?) + 108,
                ))?,
                251..=254 => self.push(f32::from(
                    -(i16::from(op) - 251) * 256 - i16::from(r.u8()?) - 108,
                ))?,
                28 => self.push(f32::from(r.i16()?))?,
                255 => self.push(r.i32()? as f32 / 65536.0)?,
                1 | 3 | 18 | 23 => {
                    self.take_width(true);
                    self.stems += (self.len / 2) as u32;
                    self.len = 0;
                }
                19 | 20 => {
                    self.take_width(true);
                    self.stems += (self.len / 2) as u32;
                    self.len = 0;
                    r.skip(u64::from(self.stems.div_ceil(8)))?;
                }
                21 => {
                    self.take_width(true);
                    if self.len != 2 {
                        return Err(malformed("CFF rmoveto arguments"));
                    }
                    self.move_to(self.arg(0), self.arg(1));
                    self.len = 0;
                }
                22 | 4 => {
                    self.take_width(false);
                    if self.len != 1 {
                        return Err(malformed("CFF hmoveto / vmoveto arguments"));
                    }
                    let v = self.arg(0);
                    if op == 22 {
                        self.move_to(v, 0.0)
                    } else {
                        self.move_to(0.0, v)
                    }
                    self.len = 0;
                }
                5 => {
                    self.need_move()?;
                    if self.len % 2 != 0 {
                        return Err(malformed("CFF rlineto arguments"));
                    }
                    for i in (0..self.len).step_by(2) {
                        self.line(self.arg(i), self.arg(i + 1));
                    }
                    self.len = 0;
                }
                6 | 7 => {
                    self.need_move()?;
                    if self.len == 0 {
                        return Err(malformed("CFF hlineto / vlineto arguments"));
                    }
                    let mut horizontal = op == 6;
                    for i in 0..self.len {
                        let v = self.arg(i);
                        if horizontal {
                            self.line(v, 0.0)
                        } else {
                            self.line(0.0, v)
                        }
                        horizontal = !horizontal;
                    }
                    self.len = 0;
                }
                8 => {
                    self.need_move()?;
                    if self.len % 6 != 0 {
                        return Err(malformed("CFF rrcurveto arguments"));
                    }
                    for i in (0..self.len).step_by(6) {
                        let d = [
                            self.arg(i),
                            self.arg(i + 1),
                            self.arg(i + 2),
                            self.arg(i + 3),
                            self.arg(i + 4),
                            self.arg(i + 5),
                        ];
                        self.curve(d);
                    }
                    self.len = 0;
                }
                24 => {
                    self.need_move()?;
                    if self.len < 8 || (self.len - 2) % 6 != 0 {
                        return Err(malformed("CFF rcurveline arguments"));
                    }
                    let mut i = 0;
                    while i < self.len - 2 {
                        let d = [
                            self.arg(i),
                            self.arg(i + 1),
                            self.arg(i + 2),
                            self.arg(i + 3),
                            self.arg(i + 4),
                            self.arg(i + 5),
                        ];
                        self.curve(d);
                        i += 6;
                    }
                    self.line(self.arg(i), self.arg(i + 1));
                    self.len = 0;
                }
                25 => {
                    self.need_move()?;
                    if self.len < 8 || (self.len - 6) % 2 != 0 {
                        return Err(malformed("CFF rlinecurve arguments"));
                    }
                    let mut i = 0;
                    while i < self.len - 6 {
                        self.line(self.arg(i), self.arg(i + 1));
                        i += 2;
                    }
                    let d = [
                        self.arg(i),
                        self.arg(i + 1),
                        self.arg(i + 2),
                        self.arg(i + 3),
                        self.arg(i + 4),
                        self.arg(i + 5),
                    ];
                    self.curve(d);
                    self.len = 0;
                }
                26 | 27 => {
                    // vvcurveto: dx1? {dya dxb dyb dyc}+ / hhcurveto: dy1? {dxa dxb dyb dxc}+
                    self.need_move()?;
                    let mut i = 0;
                    if self.len % 2 == 1 {
                        if op == 26 {
                            self.x += self.arg(0)
                        } else {
                            self.y += self.arg(0)
                        }
                        i = 1;
                    }
                    if (self.len - i) % 4 != 0 {
                        return Err(malformed("CFF vvcurveto / hhcurveto arguments"));
                    }
                    while i < self.len {
                        let (a, b, c, d) = (
                            self.arg(i),
                            self.arg(i + 1),
                            self.arg(i + 2),
                            self.arg(i + 3),
                        );
                        if op == 26 {
                            self.curve([0.0, a, b, c, 0.0, d])
                        } else {
                            self.curve([a, 0.0, b, c, d, 0.0])
                        }
                        i += 4;
                    }
                    self.len = 0;
                }
                30 | 31 => self.alternating_curves(op == 31)?,
                10 | 29 => {
                    let n = self.pop()?;
                    let subrs = if op == 10 {
                        self.locals
                            .ok_or(malformed("CFF callsubr without local subrs"))?
                    } else {
                        self.cff.globals
                    };
                    let i = i64::from(n as i32) + bias(subrs.count);
                    if !(0..i64::from(subrs.count)).contains(&i) || n != (n as i32) as f32 {
                        return Err(malformed("CFF subroutine index"));
                    }
                    let body = subrs.get(i as u32)?;
                    self.run(body, depth + 1)?;
                    if self.ended {
                        return Ok(());
                    }
                }
                11 => {
                    if self.cff.cff2 {
                        return Err(malformed("CFF2 has no return operator"));
                    }
                    return Ok(());
                }
                14 => {
                    if self.cff.cff2 {
                        return Err(malformed("CFF2 has no endchar operator"));
                    }
                    self.take_width(true);
                    if self.len == 4 {
                        return Err(unsupported(
                            "CFF endchar with seac (accented glyph by standard encoding)",
                        ));
                    }
                    if self.len != 0 {
                        return Err(malformed("CFF endchar arguments"));
                    }
                    self.ended = true;
                    return Ok(());
                }
                15 if self.cff.cff2 => {
                    self.vsindex = self.pop()? as u32;
                    self.len = 0;
                }
                16 if self.cff.cff2 => {
                    // blend at the default instance: n defaults followed by n * k deltas; keep the defaults
                    let n = self.pop()?;
                    if n < 0.0 || n != (n as u32) as f32 {
                        return Err(malformed("CFF2 blend count"));
                    }
                    let k = u64::from(self.cff.region_count(self.vsindex)?);
                    let n = n as u64;
                    let deltas = n.checked_mul(k).ok_or(malformed("CFF2 blend operands"))?;
                    if n + deltas > self.len as u64 {
                        return Err(malformed("CFF2 blend operands"));
                    }
                    self.len -= deltas as usize;
                }
                12 => {
                    let op2 = r.u8()?;
                    self.need_move()?;
                    let a: [f32; 13] =
                        core::array::from_fn(|i| if i < self.len { self.arg(i) } else { 0.0 });
                    match (op2, self.len) {
                        (35, 13) => {
                            self.curve([a[0], a[1], a[2], a[3], a[4], a[5]]);
                            self.curve([a[6], a[7], a[8], a[9], a[10], a[11]]);
                        }
                        (34, 7) => {
                            let y0 = self.y;
                            self.curve([a[0], 0.0, a[1], a[2], a[3], 0.0]);
                            self.curve([a[4], 0.0, a[5], y0 - self.y, a[6], 0.0]);
                            self.y = y0;
                        }
                        (36, 9) => {
                            let y0 = self.y;
                            self.curve([a[0], a[1], a[2], a[3], a[4], 0.0]);
                            let x4 = self.x + a[5];
                            let x5 = x4 + a[6];
                            let y5 = self.y + a[7];
                            self.sink.curve_to(x4, self.y, x5, y5, x5 + a[8], y0);
                            self.x = x5 + a[8];
                            self.y = y0;
                        }
                        (37, 11) => {
                            let (x0, y0) = (self.x, self.y);
                            let x1 = x0 + a[0];
                            let y1 = y0 + a[1];
                            let x2 = x1 + a[2];
                            let y2 = y1 + a[3];
                            let x3 = x2 + a[4];
                            let y3 = y2 + a[5];
                            let x4 = x3 + a[6];
                            let y4 = y3 + a[7];
                            let x5 = x4 + a[8];
                            let y5 = y4 + a[9];
                            let (x6, y6) = if abs(x5 - x0) > abs(y5 - y0) {
                                (x5 + a[10], y0)
                            } else {
                                (x0, y5 + a[10])
                            };
                            self.sink.curve_to(x1, y1, x2, y2, x3, y3);
                            self.sink.curve_to(x4, y4, x5, y5, x6, y6);
                            self.x = x6;
                            self.y = y6;
                        }
                        (34..=37, _) => return Err(malformed("CFF flex arguments")),
                        _ => return Err(unsupported("CFF escape operator")),
                    }
                    self.len = 0;
                }
                _ => return Err(malformed("CFF reserved operator")),
            }
        }
        Ok(())
    }

    /// hvcurveto (`h_first`) and vhcurveto: curves that start alternately horizontal and vertical
    fn alternating_curves(&mut self, mut h_first: bool) -> Result<()> {
        self.need_move()?;
        if self.len < 4 {
            return Err(malformed("CFF hvcurveto / vhcurveto arguments"));
        }
        let mut i = 0;
        while i < self.len {
            if self.len - i < 4 {
                return Err(malformed("CFF hvcurveto / vhcurveto arguments"));
            }
            let (a, b, c, d) = (
                self.arg(i),
                self.arg(i + 1),
                self.arg(i + 2),
                self.arg(i + 3),
            );
            let last = if self.len - i == 5 {
                self.arg(i + 4)
            } else {
                0.0
            };
            if h_first {
                self.curve([a, 0.0, b, c, last, d]);
            } else {
                self.curve([0.0, a, b, c, d, last]);
            }
            i += if self.len - i == 5 { 5 } else { 4 };
            h_first = !h_first;
        }
        self.len = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;
    use alloc::string::String;
    use alloc::vec::Vec;
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
        fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
            self.0.push_str("Q ");
        }
        fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
            let _ = write!(self.0, "C{x1},{y1},{x2},{y2},{x},{y} ");
        }
        fn close(&mut self) {
            self.0.push_str("Z ");
        }
    }

    /// an INDEX with 1-byte offsets
    fn index(entries: &[&[u8]]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(entries.len() as u16).to_be_bytes());
        if entries.is_empty() {
            return out;
        }
        out.push(1);
        let mut off = 1u8;
        out.push(off);
        for e in entries {
            off += e.len() as u8;
            out.push(off);
        }
        for e in entries {
            out.extend_from_slice(e);
        }
        out
    }

    /// a CFF table: header, Name INDEX, Top DICT INDEX (CharStrings offset), empty String INDEX, the global
    /// subroutines and the charstrings
    fn table(charstrings: &[&[u8]], globals: &[&[u8]]) -> Vec<u8> {
        let header = [1u8, 0, 4, 1];
        let name = index(&[b"x"]);
        let strings = index(&[]);
        let gsubrs = index(globals);
        // Top DICT: 29 <i32 offset> 17; its size is fixed, so the CharStrings offset can be computed first
        let top_len = index(&[&[0u8; 6]]).len();
        let cs_off = header.len() + name.len() + top_len + strings.len() + gsubrs.len();
        let mut top = alloc::vec![29u8];
        top.extend_from_slice(&(cs_off as i32).to_be_bytes());
        top.push(17);
        let mut out = header.to_vec();
        out.extend(name);
        out.extend(index(&[&top]));
        out.extend(strings);
        out.extend(gsubrs);
        out.extend(index(charstrings));
        out
    }

    fn draw(cs: &[u8], globals: &[&[u8]]) -> core::result::Result<String, ErrorKind> {
        let t = table(&[cs], globals);
        let cff = Cff::parse(&t, false, 1).map_err(|e| e.kind)?;
        let mut log = Log::default();
        cff.outline(0, &mut log).map_err(|e| e.kind)?;
        Ok(log.0)
    }

    // operands 32..=246 are v - 139
    const fn n(v: i8) -> u8 {
        (v as i16 + 139) as u8
    }

    #[test]
    fn moves_and_lines_are_relative() {
        // 10 20 rmoveto 30 0 rlineto 0 40 rlineto endchar
        let cs = [n(10), n(20), 21, n(30), n(0), 5, n(0), n(40), 5, 14];
        assert_eq!(draw(&cs, &[]).unwrap(), "M10,20 L40,20 L40,60 Z ");
    }

    #[test]
    fn a_leading_width_operand_is_dropped() {
        // width 50, then 10 20 rmoveto endchar
        let cs = [n(50), n(10), n(20), 21, 14];
        assert_eq!(draw(&cs, &[]).unwrap(), "M10,20 Z ");
    }

    #[test]
    fn more_than_48_operands_is_a_limit() {
        let mut cs = alloc::vec![n(1); 49];
        cs.push(14);
        assert_eq!(draw(&cs, &[]).unwrap_err(), ErrorKind::Limit);
        let mut ok = alloc::vec![n(1); 48];
        ok.extend_from_slice(&[5, 14]); // rlineto before a moveto
        assert_eq!(draw(&ok, &[]).unwrap_err(), ErrorKind::Malformed);
    }

    /// global subr k calls subr k + 1 for k < n - 1; the charstring calls subr 0: n nested calls
    fn nested(calls: usize) -> core::result::Result<String, ErrorKind> {
        let bodies: Vec<Vec<u8>> = (0..calls)
            .map(|k| {
                if k + 1 < calls {
                    alloc::vec![n((k as i16 + 1 - 107) as i8), 29, 11]
                } else {
                    alloc::vec![11]
                }
            })
            .collect();
        let refs: Vec<&[u8]> = bodies.iter().map(|b| b.as_slice()).collect();
        draw(&[n(10), n(10), 21, n(-107), 29, 14], &refs)
    }

    #[test]
    fn subroutines_nest_up_to_the_depth_limit() {
        let limit = usize::from(MAX_SUBR_DEPTH);
        assert_eq!(nested(limit).unwrap(), "M10,10 Z ");
        assert_eq!(nested(limit + 1).unwrap_err(), ErrorKind::Limit);
    }

    #[test]
    fn a_subroutine_that_calls_itself_hits_the_depth_limit() {
        // global subr 0 (index -107 with bias 107): -107 callgsubr
        let g: &[u8] = &[n(-107), 29];
        let cs = [n(-107), 29, 14];
        assert_eq!(draw(&cs, &[g]).unwrap_err(), ErrorKind::Limit);
    }

    #[test]
    fn a_subroutine_index_outside_the_index_is_an_error() {
        let g: &[u8] = &[n(10), n(10), 21, 11];
        assert_eq!(draw(&[n(-107), 29, 14], &[g]).unwrap(), "M10,10 Z ");
        assert_eq!(
            draw(&[n(-106), 29, 14], &[g]).unwrap_err(),
            ErrorKind::Malformed
        );
        assert_eq!(
            draw(&[n(-107), 10, 14], &[g]).unwrap_err(),
            ErrorKind::Malformed
        ); // no local subrs
    }

    #[test]
    fn a_hint_mask_past_the_charstring_is_truncated() {
        // 2 stems (4 operands) then hintmask needs 1 mask byte, which is missing
        let cs = [n(0), n(10), n(20), n(10), 18, 19];
        assert_eq!(draw(&cs, &[]).unwrap_err(), ErrorKind::Truncated);
        let cs = [n(0), n(10), n(20), n(10), 18, 19, 0xC0, n(5), n(5), 21, 14];
        assert_eq!(draw(&cs, &[]).unwrap(), "M5,5 Z ");
    }

    #[test]
    fn a_charstring_without_endchar_is_an_error() {
        assert_eq!(
            draw(&[n(10), n(20), 21], &[]).unwrap_err(),
            ErrorKind::Malformed
        );
    }

    #[test]
    fn hvcurveto_alternates_and_takes_a_last_operand() {
        // 0 0 rmoveto; 10 20 30 40 5 hvcurveto: horizontal start, vertical end, extra dx 5
        let cs = [n(0), n(0), 21, n(10), n(20), n(30), n(40), n(5), 31, 14];
        assert_eq!(draw(&cs, &[]).unwrap(), "M0,0 C10,0,30,30,35,70 Z ");
    }

    #[test]
    fn an_index_whose_offsets_run_past_the_table_is_an_error() {
        let mut t = table(&[&[n(1), n(1), 21, 14]], &[]);
        let last = t.len() - 5; // the CharStrings INDEX's last offset byte
        t[last] = 200;
        assert_eq!(
            Cff::parse(&t, false, 1).err().unwrap().kind,
            ErrorKind::Truncated
        );
    }
}

//! `cmap`: code point → glyph id (formats 0, 4, 6, 10, 12, 13) and variation sequences (format 14).
//!
//! Every subtable's declared counts are checked against its bytes when the font is opened, so a lookup only
//! reads inside arrays whose size is already known to fit.

use crate::error::{malformed, unsupported, Result};
use crate::reader::{slice, tail, u16_at, Reader};
use alloc::vec::Vec;

/// A Unicode subtable that maps code points to glyph ids.
#[derive(Clone, Copy)]
enum Map<'a> {
    /// format 0: 256 glyph ids, one byte each
    Byte(&'a [u8]),
    /// format 4: segment arrays and the glyph id array that follows them
    Segments {
        seg_count: usize,
        end: &'a [u8],
        start: &'a [u8],
        delta: &'a [u8],
        range: &'a [u8],
        range_and_glyphs: &'a [u8],
    },
    /// format 6 (16-bit) and 10 (32-bit): a run of glyph ids from the first code
    Trimmed { first: u32, glyphs: &'a [u8] },
    /// format 12 (`many_to_one == false`) and 13 (`true`): (start, end, glyph) groups
    Groups { groups: &'a [u8], many_to_one: bool },
}

/// The result of a variation sequence lookup (format 14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variation {
    /// The sequence uses the glyph the base code point maps to.
    UseDefault,
    /// The sequence maps to this glyph.
    Found(u16),
}

#[derive(Clone)]
pub(crate) struct Cmap<'a> {
    maps: Vec<Map<'a>>,
    variations: Option<&'a [u8]>,
}

fn is_unicode(platform: u16, encoding: u16, format: u16) -> bool {
    match platform {
        0 => true,
        3 => encoding == 1 || (encoding == 10 && (format == 12 || format == 13)),
        _ => false,
    }
}

impl<'a> Cmap<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Result<Self> {
        let mut r = Reader::new(data, "cmap header");
        let _version = r.u16()?;
        let count = r.u16()?;
        let records = r.array(u64::from(count), 8)?;
        let mut out = Cmap {
            maps: Vec::new(),
            variations: None,
        };
        for i in 0..usize::from(count) {
            let mut rec = Reader::new(&records[i * 8..i * 8 + 8], "cmap encoding record");
            let platform = rec.u16()?;
            let encoding = rec.u16()?;
            let offset = rec.u32()?;
            let sub = tail(data, u64::from(offset), "cmap subtable offset")?;
            let format = Reader::new(sub, "cmap subtable format").u16()?;
            if format == 14 {
                if platform == 0 && encoding == 5 && out.variations.is_none() {
                    out.variations = Some(parse_format14(sub)?);
                }
                continue;
            }
            if !is_unicode(platform, encoding, format) {
                continue;
            }
            let map = match format {
                0 => parse_format0(sub)?,
                4 => parse_format4(sub)?,
                6 => parse_format6(sub)?,
                10 => parse_format10(sub)?,
                12 | 13 => parse_groups(sub, format == 13)?,
                // format 2 (mixed 8/16-bit) and 8 are not Unicode-compatible in practice; skipped
                2 | 8 => continue,
                _ => return Err(unsupported("cmap subtable format")),
            };
            out.maps.push(map);
        }
        Ok(out)
    }

    /// The glyph for a code point: the first Unicode subtable (in record order) that maps it to a non-zero id.
    pub(crate) fn glyph(&self, cp: u32) -> Result<Option<u16>> {
        for map in &self.maps {
            if let Some(g) = lookup(map, cp)? {
                return Ok(Some(g));
            }
        }
        Ok(None)
    }

    pub(crate) fn variation(&self, cp: u32, selector: u32) -> Result<Option<Variation>> {
        match self.variations {
            Some(sub) => lookup_format14(sub, cp, selector),
            None => Ok(None),
        }
    }
}

/// the subtable's bytes as its length field declares them (the length must fit in what is there)
fn sub_with_len<'d>(sub: &'d [u8], len: u64, context: &'static str) -> Result<&'d [u8]> {
    slice(sub, 0, len, context)
}

fn parse_format0(sub: &[u8]) -> Result<Map<'_>> {
    let mut r = Reader::new(sub, "cmap format 0");
    r.skip(6)?; // format, length, language
    Ok(Map::Byte(r.bytes(256)?))
}

fn parse_format4(sub: &[u8]) -> Result<Map<'_>> {
    let mut r = Reader::new(sub, "cmap format 4");
    r.skip(2)?;
    let len = u64::from(r.u16()?);
    // some fonts declare a length past the subtable's real end; read what is there, never more than declared
    let body = sub_with_len(sub, len.min(sub.len() as u64), "cmap format 4")?;
    let mut r = Reader::new(body, "cmap format 4");
    r.skip(6)?;
    let seg_x2 = r.u16()?;
    if seg_x2 % 2 != 0 {
        return Err(malformed("cmap format 4 segCountX2"));
    }
    let seg_count = usize::from(seg_x2 / 2);
    r.skip(6)?; // searchRange, entrySelector, rangeShift
    let end = r.array(seg_count as u64, 2)?;
    r.skip(2)?; // reservedPad
    let start = r.array(seg_count as u64, 2)?;
    let delta = r.array(seg_count as u64, 2)?;
    let range_and_glyphs = &body[body.len() - r.remaining()..];
    let range =
        Reader::new(range_and_glyphs, "cmap format 4 idRangeOffset").array(seg_count as u64, 2)?;
    Ok(Map::Segments {
        seg_count,
        end,
        start,
        delta,
        range,
        range_and_glyphs,
    })
}

fn parse_format6(sub: &[u8]) -> Result<Map<'_>> {
    let mut r = Reader::new(sub, "cmap format 6");
    r.skip(6)?;
    let first = u32::from(r.u16()?);
    let count = r.u16()?;
    Ok(Map::Trimmed {
        first,
        glyphs: r.array(u64::from(count), 2)?,
    })
}

fn parse_format10(sub: &[u8]) -> Result<Map<'_>> {
    let mut r = Reader::new(sub, "cmap format 10");
    r.skip(12)?; // format, reserved, length, language
    let first = r.u32()?;
    let count = r.u32()?;
    Ok(Map::Trimmed {
        first,
        glyphs: r.array(u64::from(count), 2)?,
    })
}

fn parse_groups(sub: &[u8], many_to_one: bool) -> Result<Map<'_>> {
    let mut r = Reader::new(sub, "cmap format 12 / 13");
    r.skip(12)?;
    let count = r.u32()?;
    Ok(Map::Groups {
        groups: r.array(u64::from(count), 12)?,
        many_to_one,
    })
}

fn nonzero(g: u32) -> Option<u16> {
    match u16::try_from(g) {
        Ok(0) | Err(_) => None,
        Ok(g) => Some(g),
    }
}

fn lookup(map: &Map<'_>, cp: u32) -> Result<Option<u16>> {
    match *map {
        Map::Byte(ids) => Ok(usize::try_from(cp)
            .ok()
            .and_then(|i| ids.get(i))
            .and_then(|&g| nonzero(u32::from(g)))),
        Map::Segments {
            seg_count,
            end,
            start,
            delta,
            range,
            range_and_glyphs,
        } => {
            let Ok(cp) = u16::try_from(cp) else {
                return Ok(None);
            };
            for i in 0..seg_count {
                let seg_end = u16_at(end, i, "cmap format 4 endCode")?;
                if cp > seg_end {
                    continue;
                }
                let seg_start = u16_at(start, i, "cmap format 4 startCode")?;
                if cp < seg_start {
                    return Ok(None);
                }
                let d = u16_at(delta, i, "cmap format 4 idDelta")?;
                let ro = u16_at(range, i, "cmap format 4 idRangeOffset")?;
                if ro == 0 {
                    return Ok(nonzero(u32::from(cp.wrapping_add(d))));
                }
                if ro == 0xFFFF {
                    return Ok(None);
                }
                // the glyph id lives `ro` bytes after this idRangeOffset entry, indexed by cp - start
                let at = (i as u64) * 2 + u64::from(ro) + 2 * u64::from(cp - seg_start);
                let b = slice(range_and_glyphs, at, 2, "cmap format 4 glyphIdArray")?;
                let g = u16::from_be_bytes([b[0], b[1]]);
                if g == 0 {
                    return Ok(None);
                }
                return Ok(nonzero(u32::from(g.wrapping_add(d))));
            }
            Ok(None)
        }
        Map::Trimmed { first, glyphs } => {
            let Some(i) = cp.checked_sub(first) else {
                return Ok(None);
            };
            if u64::from(i) >= glyphs.len() as u64 / 2 {
                return Ok(None);
            }
            Ok(nonzero(u32::from(u16_at(
                glyphs,
                i as usize,
                "cmap glyph array",
            )?)))
        }
        Map::Groups {
            groups,
            many_to_one,
        } => {
            for g in groups.chunks_exact(12) {
                let mut r = Reader::new(g, "cmap group");
                let (s, e, glyph) = (r.u32()?, r.u32()?, r.u32()?);
                if cp >= s && cp <= e {
                    let id = if many_to_one {
                        Some(glyph)
                    } else {
                        glyph.checked_add(cp - s)
                    };
                    return Ok(id.and_then(nonzero));
                }
            }
            Ok(None)
        }
    }
}

fn parse_format14(sub: &[u8]) -> Result<&[u8]> {
    let mut r = Reader::new(sub, "cmap format 14");
    r.skip(2)?;
    let len = r.u32()?;
    let body = slice(sub, 0, u64::from(len), "cmap format 14 length")?;
    let mut r = Reader::new(body, "cmap format 14");
    r.skip(6)?;
    let count = r.u32()?;
    r.array(u64::from(count), 11)?;
    // each record's tables are checked when they are read (the offsets are relative to the subtable)
    Ok(body)
}

fn lookup_format14(sub: &[u8], cp: u32, selector: u32) -> Result<Option<Variation>> {
    let mut r = Reader::new(sub, "cmap format 14");
    r.skip(6)?;
    let count = r.u32()?;
    let records = r.array(u64::from(count), 11)?;
    for rec in records.chunks_exact(11) {
        let mut rr = Reader::new(rec, "cmap format 14 record");
        let vs = rr.u24()?;
        let default_off = rr.u32()?;
        let non_default_off = rr.u32()?;
        if vs != selector {
            continue;
        }
        if non_default_off != 0 {
            let mut t = Reader::at(
                sub,
                u64::from(non_default_off),
                "cmap format 14 non-default UVS",
            )?;
            let n = t.u32()?;
            let maps = t.array(u64::from(n), 5)?;
            for m in maps.chunks_exact(5) {
                let mut mr = Reader::new(m, "cmap format 14 UVS mapping");
                if mr.u24()? == cp {
                    return Ok(Some(Variation::Found(mr.u16()?)));
                }
            }
        }
        if default_off != 0 {
            let mut t = Reader::at(sub, u64::from(default_off), "cmap format 14 default UVS")?;
            let n = t.u32()?;
            let ranges = t.array(u64::from(n), 4)?;
            for rg in ranges.chunks_exact(4) {
                let mut rr = Reader::new(rg, "cmap format 14 unicode range");
                let start = rr.u24()?;
                let extra = u32::from(rr.u8()?);
                if cp >= start && cp <= start + extra {
                    return Ok(Some(Variation::UseDefault));
                }
            }
        }
        return Ok(None);
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    fn cmap_with(platform: u16, encoding: u16, sub: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&1u16.to_be_bytes());
        out.extend_from_slice(&platform.to_be_bytes());
        out.extend_from_slice(&encoding.to_be_bytes());
        out.extend_from_slice(&12u32.to_be_bytes());
        out.extend_from_slice(sub);
        out
    }

    /// the input that reached ttf-parser's debug_assert: a format 10 count far past the data
    #[test]
    fn a_format_10_count_past_the_data_is_an_error() {
        let mut sub = Vec::new();
        sub.extend_from_slice(&10u16.to_be_bytes());
        sub.extend_from_slice(&[0, 0]);
        sub.extend_from_slice(&0u32.to_be_bytes());
        sub.extend_from_slice(&0u32.to_be_bytes());
        sub.extend_from_slice(&0x41u32.to_be_bytes());
        sub.extend_from_slice(&0xFFFF_FFFFu32.to_be_bytes());
        sub.extend_from_slice(&7u16.to_be_bytes());
        let data = cmap_with(0, 4, &sub);
        let e = Cmap::parse(&data).err().expect("rejected");
        assert_eq!(
            (e.kind, e.context),
            (ErrorKind::Truncated, "cmap format 10")
        );
    }

    #[test]
    fn format_10_and_12_and_13_map() {
        let mut f10 = Vec::new();
        f10.extend_from_slice(&10u16.to_be_bytes());
        f10.extend_from_slice(&[0; 10]);
        f10.extend_from_slice(&0x1F600u32.to_be_bytes());
        f10.extend_from_slice(&2u32.to_be_bytes());
        f10.extend_from_slice(&5u16.to_be_bytes());
        f10.extend_from_slice(&0u16.to_be_bytes());
        let data = cmap_with(0, 4, &f10);
        let c = Cmap::parse(&data).unwrap();
        assert_eq!(c.glyph(0x1F600).unwrap(), Some(5));
        assert_eq!(c.glyph(0x1F601).unwrap(), None); // glyph 0 is "missing"
        assert_eq!(c.glyph(0x1F602).unwrap(), None);
        assert_eq!(c.glyph(0x41).unwrap(), None);

        for (format, want) in [(12u16, [Some(10), Some(12)]), (13, [Some(10), Some(10)])] {
            let mut g = Vec::new();
            g.extend_from_slice(&format.to_be_bytes());
            g.extend_from_slice(&[0; 10]);
            g.extend_from_slice(&1u32.to_be_bytes());
            for v in [0x100u32, 0x102, 10] {
                g.extend_from_slice(&v.to_be_bytes());
            }
            let data = cmap_with(3, 10, &g);
            let c = Cmap::parse(&data).unwrap();
            assert_eq!(
                [c.glyph(0x100).unwrap(), c.glyph(0x102).unwrap()],
                want,
                "format {format}"
            );
            assert_eq!(c.glyph(0x103).unwrap(), None);
        }
    }

    #[test]
    fn a_non_unicode_subtable_is_not_used() {
        let mut f6 = Vec::new();
        f6.extend_from_slice(&6u16.to_be_bytes());
        f6.extend_from_slice(&[0; 4]);
        f6.extend_from_slice(&0x41u16.to_be_bytes());
        f6.extend_from_slice(&1u16.to_be_bytes());
        f6.extend_from_slice(&3u16.to_be_bytes());
        assert_eq!(
            Cmap::parse(&cmap_with(1, 0, &f6))
                .unwrap()
                .glyph(0x41)
                .unwrap(),
            None
        );
        assert_eq!(
            Cmap::parse(&cmap_with(3, 1, &f6))
                .unwrap()
                .glyph(0x41)
                .unwrap(),
            Some(3)
        );
    }

    #[test]
    fn a_record_offset_past_the_table_is_an_error() {
        let mut data = cmap_with(3, 1, &[]);
        data[8..12].copy_from_slice(&1000u32.to_be_bytes());
        assert_eq!(Cmap::parse(&data).err().unwrap().kind, ErrorKind::Truncated);
    }
}

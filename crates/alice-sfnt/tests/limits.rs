//! Hostile fonts that are small but make a reader do unbounded work: every one must stop early with an
//! error (or answer quickly), never run for seconds or grow without bound.
//! oracle: the work each font asks for is computed from its construction (2^depth component leaves, records ×
//! segments lookups); the limits are the crate's MAX_GLYF_WORK / MAX_CMAP_SUBTABLES.

use alice_sfnt::{ErrorKind, Font, OutlineSink};

/// counts segments without storing them (no allocation per segment)
#[derive(Default)]
struct Count(u64);

impl OutlineSink for Count {
    fn move_to(&mut self, _: f32, _: f32) {
        self.0 += 1;
    }
    fn line_to(&mut self, _: f32, _: f32) {
        self.0 += 1;
    }
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 1;
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 1;
    }
    fn close(&mut self) {}
}

/// an sfnt with the given tables (tag, bytes), table directory sorted by tag
fn sfnt(mut tables: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    tables.sort_by_key(|t| t.0);
    let mut out = Vec::new();
    out.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    out.extend_from_slice(&(tables.len() as u16).to_be_bytes());
    out.extend_from_slice(&[0; 6]);
    let mut offset = 12 + 16 * tables.len();
    for (tag, data) in &tables {
        out.extend_from_slice(tag);
        out.extend_from_slice(&0u32.to_be_bytes());
        out.extend_from_slice(&(offset as u32).to_be_bytes());
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        offset += data.len().next_multiple_of(4);
    }
    for (_, data) in &tables {
        out.extend_from_slice(data);
        out.resize(out.len().next_multiple_of(4), 0);
    }
    out
}

fn head(long_loca: bool) -> Vec<u8> {
    let mut h = vec![0u8; 54];
    h[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    h[12..16].copy_from_slice(&0x5F0F_3CF5u32.to_be_bytes());
    h[18..20].copy_from_slice(&1000u16.to_be_bytes());
    h[50..52].copy_from_slice(&i16::from(long_loca).to_be_bytes());
    h
}

fn maxp(glyphs: u16) -> Vec<u8> {
    let mut m = 0x0000_5000u32.to_be_bytes().to_vec();
    m.extend_from_slice(&glyphs.to_be_bytes());
    m
}

fn hhea() -> Vec<u8> {
    let mut h = vec![0u8; 36];
    h[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    h[34..36].copy_from_slice(&1u16.to_be_bytes());
    h
}

/// glyph k (k < depth) is a composite of `fan` components that all name glyph k + 1; glyph `depth` is a triangle.
/// Drawing glyph 0 asks for fan^depth triangles (fc's hostile font with fan 2).
fn fan_out(depth: u16, fan: u16) -> Vec<u8> {
    let mut glyphs: Vec<Vec<u8>> = Vec::new();
    for k in 0..depth {
        let mut c = Vec::new();
        c.extend_from_slice(&(-1i16).to_be_bytes());
        c.extend_from_slice(&[0; 8]);
        for i in 0..fan {
            let more = if i + 1 < fan { 0x0020u16 } else { 0 };
            c.extend_from_slice(&(0x0002 | more).to_be_bytes());
            c.extend_from_slice(&(k + 1).to_be_bytes());
            c.extend_from_slice(&[0, 0]);
        }
        glyphs.push(c);
    }
    let mut tri = Vec::new();
    tri.extend_from_slice(&1i16.to_be_bytes());
    tri.extend_from_slice(&[0; 8]);
    tri.extend_from_slice(&2u16.to_be_bytes()); // end point of the contour
    tri.extend_from_slice(&0u16.to_be_bytes()); // no instructions
    tri.extend_from_slice(&[0x01 | 0x02 | 0x04 | 0x10 | 0x20; 3]); // on curve, short positive x / y
    tri.extend_from_slice(&[0, 10, 5]);
    tri.extend_from_slice(&[0, 0, 8]);
    glyphs.push(tri);
    let mut loca = Vec::new();
    let mut glyf = Vec::new();
    for g in &glyphs {
        loca.extend_from_slice(&(glyf.len() as u32).to_be_bytes());
        glyf.extend_from_slice(g);
        glyf.resize(glyf.len().next_multiple_of(2), 0);
    }
    loca.extend_from_slice(&(glyf.len() as u32).to_be_bytes());
    let mut hmtx = 500u16.to_be_bytes().to_vec();
    hmtx.extend_from_slice(&[0, 0]);
    sfnt(vec![
        (*b"head", head(true)),
        (*b"hhea", hhea()),
        (*b"maxp", maxp(depth + 1)),
        (*b"hmtx", hmtx),
        (*b"loca", loca),
        (*b"glyf", glyf),
    ])
}

#[test]
fn an_exponential_composite_fan_out_stops_at_the_work_budget() {
    // depth 24, fan 2: 2^24 = 16.7M triangles asked for, from a file under 1 KB
    let data = fan_out(24, 2);
    assert!(data.len() < 1024, "{}", data.len());
    let font = Font::parse(&data, 0).unwrap();
    let mut sink = Count::default();
    let e = font.outline(0, &mut sink).unwrap_err();
    assert_eq!(e.kind, ErrorKind::Limit);
    assert!(
        sink.0 <= u64::from(alice_sfnt::MAX_GLYF_WORK) * 4,
        "{} segments before the stop",
        sink.0
    );
}

#[test]
fn a_wide_fan_out_stops_too() {
    // fan 6 at depth 10: 6^10 = 60M leaves
    let data = fan_out(10, 6);
    let font = Font::parse(&data, 0).unwrap();
    assert_eq!(
        font.outline(0, &mut Count::default()).unwrap_err().kind,
        ErrorKind::Limit
    );
}

#[test]
fn a_small_fan_out_within_the_budget_draws() {
    // 2^8 = 256 triangles, 4 segments each
    let data = fan_out(8, 2);
    let font = Font::parse(&data, 0).unwrap();
    let mut sink = Count::default();
    assert!(font.outline(0, &mut sink).unwrap());
    assert_eq!(sink.0, 256 * 4);
}

/// a cmap with `records` (3,1) encoding records that all point at one format 4 subtable of `segments` segments
fn many_records(records: u16, segments: u16, distinct: bool) -> Vec<u8> {
    let seg_x2 = segments * 2;
    let mut sub = Vec::new();
    sub.extend_from_slice(&4u16.to_be_bytes());
    let len = 16 + 8 * u32::from(segments);
    sub.extend_from_slice(&(len.min(0xFFFF) as u16).to_be_bytes());
    sub.extend_from_slice(&0u16.to_be_bytes());
    sub.extend_from_slice(&seg_x2.to_be_bytes());
    sub.extend_from_slice(&[0; 6]);
    for i in 0..segments {
        // segments in endCode order; the last is the 0xFFFF terminator
        let end = if i + 1 == segments { 0xFFFF } else { i };
        sub.extend_from_slice(&end.to_be_bytes());
    }
    sub.extend_from_slice(&[0, 0]);
    for i in 0..segments {
        let start = if i + 1 == segments { 0xFFFF } else { i };
        sub.extend_from_slice(&start.to_be_bytes());
    }
    for _ in 0..segments {
        sub.extend_from_slice(&1u16.to_be_bytes()); // idDelta: cp + 1
    }
    for _ in 0..segments {
        sub.extend_from_slice(&0u16.to_be_bytes());
    }
    let mut cmap = Vec::new();
    cmap.extend_from_slice(&0u16.to_be_bytes());
    cmap.extend_from_slice(&records.to_be_bytes());
    let base = 4 + 8 * u32::from(records);
    for r in 0..records {
        cmap.extend_from_slice(&3u16.to_be_bytes());
        cmap.extend_from_slice(&1u16.to_be_bytes());
        // distinct: each record points at its own copy (the copies follow one another)
        let off = if distinct {
            base + u32::from(r) * sub.len() as u32
        } else {
            base
        };
        cmap.extend_from_slice(&off.to_be_bytes());
    }
    let copies = if distinct { records } else { 1 };
    for _ in 0..copies {
        cmap.extend_from_slice(&sub);
    }
    let mut hmtx = 500u16.to_be_bytes().to_vec();
    hmtx.extend_from_slice(&[0, 0]);
    sfnt(vec![
        (*b"head", head(false)),
        (*b"hhea", hhea()),
        (*b"maxp", maxp(0xFFFF)),
        (*b"hmtx", hmtx),
        (*b"cmap", cmap),
    ])
}

#[test]
fn many_records_to_one_subtable_look_up_in_log_time() {
    // fc's font: 65 535 records naming one format 4 subtable of 8 000 segments (parse reads the
    // subtable once, so this is a single map). The lookup binary-searches the segments: each one
    // costs O(log segments) comparisons, not O(segments). A lookup that degraded to a linear scan
    // of the segments, or a cmap that stopped de-duplicating the records (one search per record),
    // would need far more comparisons and fail this bound; a miss within the u16 range (9000)
    // accumulates comparisons across every map tried, so it catches the de-duplication case too.
    let segments: u32 = 8000;
    let data = many_records(0xFFFF, segments as u16, false);
    let font = Font::parse(&data, 0).unwrap();
    // 2 * ceil(log2(segments)): 26 for 8000, far below the 8000 a linear scan would take
    let bound = 2 * (u32::BITS - (segments - 1).leading_zeros());
    for cp in [0x41u32, 7999, 9000, 0x10000] {
        let (_, probes) = font.glyph_index_probes(cp).unwrap();
        assert!(
            probes <= bound,
            "{probes} comparisons for cp {cp:#x} over {segments} segments (bound {bound})"
        );
    }
    assert_eq!(font.glyph_index(5).unwrap(), Some(6));
}

#[test]
fn too_many_distinct_subtables_are_refused() {
    let n = u16::try_from(alice_sfnt::MAX_CMAP_SUBTABLES).unwrap() + 1;
    let data = many_records(n, 4, true);
    let e = Font::parse(&data, 0).err().expect("refused");
    assert_eq!(e.kind, ErrorKind::Limit);
    let ok = many_records(n - 1, 4, true);
    assert!(Font::parse(&ok, 0).is_ok());
}

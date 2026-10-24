//! Outlines, lookups and metrics of the test fonts against the shapes tests/fonts/make_test_fonts.py draws.
//! oracle: the drawn points, with the TrueType rule (an off-curve point between two on-curve points is the
//! control point of one quadratic) and the CFF rule (the charstring's moves, lines and cubics in absolute
//! coordinates); no value here comes from alice-sfnt itself.

use alice_sfnt::{fonts_in, Contours, ErrorKind, Font, Segment, Variation};
use Segment::{Cubic as C, Line as L, Move as M, Quad as Q};

const QUAD: &[u8] = include_bytes!("fonts/quad.ttf");
const CUBIC: &[u8] = include_bytes!("fonts/cubic.otf");
const CUBIC2: &[u8] = include_bytes!("fonts/cubic2.otf");
const PAIR: &[u8] = include_bytes!("fonts/pair.ttc");

fn outline(font: &Font<'_>, gid: u16) -> (bool, Vec<Vec<Segment>>) {
    let mut c = Contours::default();
    let drew = font.outline(gid, &mut c).expect("the glyph reads");
    (drew, c.contours)
}

fn tri_quad() -> Vec<Segment> {
    vec![M(20.0, 0.0), L(300.0, 700.0), L(580.0, 0.0), L(20.0, 0.0)]
}

fn o_quad() -> Vec<Segment> {
    vec![
        M(350.0, 0.0),
        Q(660.0, 0.0, 660.0, 350.0),
        Q(660.0, 700.0, 350.0, 700.0),
        Q(40.0, 700.0, 40.0, 350.0),
        Q(40.0, 0.0, 350.0, 0.0),
    ]
}

fn o_cubic() -> Vec<Segment> {
    vec![
        M(350.0, 0.0),
        C(520.0, 0.0, 660.0, 160.0, 660.0, 350.0),
        C(660.0, 540.0, 520.0, 700.0, 350.0, 700.0),
        C(180.0, 700.0, 40.0, 540.0, 40.0, 350.0),
        C(40.0, 160.0, 180.0, 0.0, 350.0, 0.0),
    ]
}

#[test]
fn truetype_outlines_are_the_drawn_shapes() {
    let f = Font::parse(QUAD, 0).unwrap();
    assert_eq!((f.units_per_em(), f.num_glyphs()), (1000, 5));
    assert_eq!(outline(&f, 1), (true, vec![tri_quad()]));
    assert_eq!(outline(&f, 2), (true, vec![o_quad()]));
    assert_eq!(outline(&f, 3), (false, vec![]));
    // comp = A scaled by 1/2 and moved by (100, 50), then O
    let half = vec![
        M(110.0, 50.0),
        L(250.0, 400.0),
        L(390.0, 50.0),
        L(110.0, 50.0),
    ];
    assert_eq!(outline(&f, 4), (true, vec![half, o_quad()]));
}

#[test]
fn cff_and_cff2_outlines_are_the_drawn_shapes() {
    // CFF contours close implicitly: no segment back to the start
    let tri = vec![M(20.0, 0.0), L(300.0, 700.0), L(580.0, 0.0)];
    for (name, data) in [("CFF", CUBIC), ("CFF2", CUBIC2)] {
        let f = Font::parse(data, 0).unwrap();
        assert_eq!(f.num_glyphs(), 4, "{name}");
        assert_eq!(outline(&f, 1), (true, vec![tri.clone()]), "{name}");
        assert_eq!(outline(&f, 2), (true, vec![o_cubic()]), "{name}");
        assert_eq!(outline(&f, 3), (false, vec![]), "{name}");
    }
}

#[test]
fn cmap_maps_bmp_supplementary_and_variation_sequences() {
    let f = Font::parse(QUAD, 0).unwrap();
    let g = |cp: u32| f.glyph_index(cp).unwrap();
    assert_eq!(
        [g('A' as u32), g('O' as u32), g(' ' as u32), g('C' as u32)],
        [Some(1), Some(2), Some(3), Some(4)]
    );
    assert_eq!(g(0x1F600), Some(2)); // format 12 only
    assert_eq!(g('B' as u32), None);
    assert_eq!(g(0x10FFFF), None);
    assert_eq!(
        f.glyph_variation('A' as u32, 0xFE00).unwrap(),
        Some(Variation::UseDefault)
    );
    assert_eq!(
        f.glyph_variation('O' as u32, 0xFE00).unwrap(),
        Some(Variation::Found(4))
    );
    assert_eq!(f.glyph_variation('A' as u32, 0xFE01).unwrap(), None);
    assert_eq!(f.glyph_variation('B' as u32, 0xFE00).unwrap(), None);
}

#[test]
fn advances_are_the_hmtx_values() {
    let f = Font::parse(QUAD, 0).unwrap();
    let a: Vec<u16> = (0..5).map(|g| f.advance(g).unwrap()).collect();
    assert_eq!(a, [500, 600, 700, 250, 800]);
    assert_eq!(f.advance(5).unwrap_err().kind, ErrorKind::BadGlyph);
}

#[test]
fn a_collection_opens_each_face() {
    assert_eq!(fonts_in(PAIR).unwrap(), 2);
    assert_eq!(fonts_in(QUAD).unwrap(), 1);
    let q = Font::parse(PAIR, 0).unwrap();
    let c = Font::parse(PAIR, 1).unwrap();
    assert_eq!(outline(&q, 2), (true, vec![o_quad()]));
    assert_eq!(outline(&c, 2), (true, vec![o_cubic()]));
    assert_eq!(
        Font::parse(PAIR, 2).err().unwrap().kind,
        ErrorKind::Malformed
    );
    assert_eq!(
        Font::parse(QUAD, 1).err().unwrap().kind,
        ErrorKind::Malformed
    );
}

#[test]
fn every_truncation_of_a_font_is_an_error_or_a_font_never_a_panic() {
    for data in [QUAD, CUBIC, CUBIC2, PAIR] {
        for len in 0..data.len() {
            let cut = &data[..len];
            if let Ok(f) = Font::parse(cut, 0) {
                for gid in 0..f.num_glyphs() {
                    let _ = f.outline(gid, &mut Contours::default());
                    let _ = f.advance(gid);
                }
                for cp in [0x20, 0x41, 0x4F, 0x1F600] {
                    let _ = f.glyph_index(cp);
                    let _ = f.glyph_variation(cp, 0xFE00);
                }
            }
        }
    }
}

/// QUAD with head.unitsPerEm set to `upem` (the head table is found through the table directory)
fn with_upem(upem: u16) -> Vec<u8> {
    let mut data = QUAD.to_vec();
    let tables = u16::from_be_bytes([data[4], data[5]]) as usize;
    let rec = (0..tables)
        .map(|i| 12 + 16 * i)
        .find(|&r| &data[r..r + 4] == b"head")
        .expect("head");
    let off = u32::from_be_bytes(data[rec + 8..rec + 12].try_into().unwrap()) as usize;
    data[off + 18..off + 20].copy_from_slice(&upem.to_be_bytes());
    data
}

#[test]
fn units_per_em_outside_16_to_16384_is_an_error() {
    for (upem, ok) in [
        (0, false),
        (15, false),
        (16, true),
        (1000, true),
        (16384, true),
        (16385, false),
    ] {
        let data = with_upem(upem);
        let r = Font::parse(&data, 0);
        assert_eq!(r.is_ok(), ok, "unitsPerEm {upem}");
        if !ok {
            assert_eq!(r.err().unwrap().kind, ErrorKind::Malformed);
        }
    }
}

#[test]
fn bad_glyph_ids_are_errors() {
    let f = Font::parse(QUAD, 0).unwrap();
    assert_eq!(
        f.outline(5, &mut Contours::default()).unwrap_err().kind,
        ErrorKind::BadGlyph
    );
    let f = Font::parse(CUBIC, 0).unwrap();
    assert_eq!(
        f.outline(4, &mut Contours::default()).unwrap_err().kind,
        ErrorKind::BadGlyph
    );
}

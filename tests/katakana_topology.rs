//! Katakana stroke topology: each skeleton's stroke count, and the cell of the em box where each stroke
//! starts and ends, held against a table written by hand from BIZ UDPGothic Regular's outline.
//!
//! How the table was written: each katakana's skeleton was drawn over BIZ UDPGothic Regular's outline
//! (the font is under the SIL OFL 1.1), and each stroke's start and end were read off
//! that overlay as one of nine cells: columns a, b, c from the left, rows 1, 2, 3 from the top, the em
//! box split in thirds. Overlay sheet of 2026-11-17, after the fixes to ク, リ, ツ, ユ, オ, カ, サ, シ,
//! ム, ヤ, ヒ, チ and ウ that the same overlay showed.
//!
//! What it sees: a missing or extra stroke, a stroke that starts or ends in the wrong part of the glyph,
//! and a stroke drawn in reverse (the start and end cells swap).
//!
//! What it cannot see: proportion and placement inside a cell, curvature, and stroke width. It is a
//! mitigation written by hand; the root check derives the skeletons from the reference outline's medial
//! axis and measures the distance to it.
//!
//! A point counts as in its cell when it lies inside the cell grown by `MARGIN` on each side, so a
//! skeleton point near a cell edge does not flip the check.

use alice_font::{glyph_dispatcher, MetaFontParams};

const MARGIN: f32 = 0.06;

/// (glyph, strokes in skeleton order: `<start cell>><end cell>` separated by spaces)
const TOPOLOGY: &[(char, &str)] = &[
    ('ア', "a1>c1 c1>b3 b2>a3"),
    ('イ', "c1>a2 b2>b3"),
    ('ウ', "b1>b1 a1>c1 a1>a2 c1>b3"),
    ('エ', "a1>c1 b1>b3 a3>c3"),
    ('オ', "a1>c1 b1>b3 b3>a3 b2>a3"),
    ('カ', "a1>c1 c1>b3 b1>a3"),
    ('キ', "a1>c1 a2>c2 b1>b3"),
    ('ク', "b1>a2 a1>c1 c1>a3"),
    ('ケ', "a1>a3 a1>c1 b1>b3"),
    ('コ', "a1>c1 c1>c3 a3>c3"),
    ('サ', "a2>c2 b1>a2 b1>b3"),
    ('シ', "a1>a1 a2>a2 a3>c2"),
    ('ス', "a1>c1 c1>a3 b2>c3"),
    ('セ', "a1>c2 b1>b3 b3>c3"),
    ('ソ', "b1>b1 c1>a3"),
    ('タ', "c1>a2 a2>c2 c2>b3 b2>c3"),
    ('チ', "c1>a1 a2>c2 b1>b3"),
    ('ツ', "a1>a2 b1>b1 c1>a3"),
    ('テ', "a1>c1 a2>c2 b2>a3"),
    ('ト', "b1>a3 b2>c3"),
    ('ナ', "a1>c1 b1>b3"),
    ('ニ', "a1>c1 a3>c3"),
    ('ヌ', "a1>c1 c1>a3 b2>c3"),
    ('ネ', "b1>b1 a1>c1 b1>b3 a2>a3 b2>c3"),
    ('ノ', "c1>a3"),
    ('ハ', "b1>a3 b1>c3"),
    ('ヒ', "a2>c1 a1>a3 a3>c3"),
    ('フ', "a1>c1 c1>a3"),
    ('ヘ', "a2>b1 b1>c2"),
    ('ホ', "a2>c2 b1>b3 b2>a3 b2>c3"),
    ('マ', "a1>c1 c1>a3 b2>b3"),
    ('ミ', "a1>b1 a2>c2 a3>c3"),
    ('ム', "b1>a3 a3>c3 b2>c3"),
    ('メ', "b1>a3 a1>c3"),
    ('モ', "a1>c1 a2>c2 b1>b3 b3>c3"),
    ('ヤ', "a2>c1 c1>c2 b1>b3"),
    ('ユ', "a1>c1 c1>c3 a3>c3"),
    ('ヨ', "a1>c1 c1>c3 a2>c2 a3>c3"),
    ('ラ', "b1>b1 a2>c2 c2>a3"),
    ('リ', "a1>a2 c1>b3"),
    ('ル', "a1>b3 b1>b3 b3>c3"),
    ('レ', "a1>b3 b3>c2"),
    ('ロ', "a1>c1 c1>c3 a3>c3 a1>a3"),
    ('ワ', "a1>c1 a1>a2 c1>c2 c2>a3"),
    ('ヲ', "a1>c1 c1>c2 a2>c2 c2>a3"),
    ('ン', "a1>a2 a3>c1"),
    ('ー', "a2>c2"),
];

/// Whether `(x, y)` (em units, y up) lies in `cell` (for example "b3") grown by `MARGIN`
fn in_cell(cell: &str, x: f32, y: f32) -> bool {
    let b = cell.as_bytes();
    let col = f32::from(b[0] - b'a');
    let row = f32::from(b[1] - b'1');
    let (x0, x1) = (col / 3.0, (col + 1.0) / 3.0);
    let (y1, y0) = (1.0 - row / 3.0, 1.0 - (row + 1.0) / 3.0);
    x >= x0 - MARGIN && x <= x1 + MARGIN && y >= y0 - MARGIN && y <= y1 + MARGIN
}

fn faults(table: &[(char, &str)]) -> (usize, Vec<String>) {
    let params = MetaFontParams::sans_regular();
    let mut compared = 0;
    let mut out = Vec::new();
    for &(ch, row) in table {
        let Some(sk) = glyph_dispatcher::skeleton(ch, &params) else {
            out.push(format!("{ch}: no skeleton"));
            continue;
        };
        let want: Vec<&str> = row.split_whitespace().collect();
        if sk.stroke_count != want.len() {
            out.push(format!(
                "{ch}: {} strokes, the table has {}",
                sk.stroke_count,
                want.len()
            ));
            continue;
        }
        for (i, (st, w)) in sk.strokes[..sk.stroke_count].iter().zip(&want).enumerate() {
            let (a, b) = w.split_once('>').expect("a table entry is <cell>><cell>");
            compared += 1;
            if !in_cell(a, st.p0.x, st.p0.y) || !in_cell(b, st.p3.x, st.p3.y) {
                out.push(format!(
                    "{ch} stroke {i}: ({:.2}, {:.2}) > ({:.2}, {:.2}), the table has {w}",
                    st.p0.x, st.p0.y, st.p3.x, st.p3.y
                ));
            }
        }
    }
    (compared, out)
}

#[test]
fn every_katakana_has_the_strokes_of_the_reference() {
    let (compared, out) = faults(TOPOLOGY);
    assert!(compared >= 100, "only {compared} strokes compared");
    assert!(out.is_empty(), "{}", out.join("\n"));
}

#[test]
fn the_table_covers_every_base_katakana() {
    let base = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲンー";
    for ch in base.chars() {
        assert!(
            TOPOLOGY.iter().any(|&(c, _)| c == ch),
            "{ch} is not in the table"
        );
    }
    assert_eq!(TOPOLOGY.len(), base.chars().count());
}

#[test]
fn a_dropped_or_reversed_stroke_is_seen() {
    // the check has teeth: a table that drops ク's first stroke, or reverses リ's right stroke, fails
    let dropped: Vec<(char, String)> = TOPOLOGY
        .iter()
        .map(|&(c, r)| {
            (
                c,
                if c == 'ク' {
                    r.split_once(' ').unwrap().1.to_string()
                } else {
                    r.to_string()
                },
            )
        })
        .collect();
    let dropped: Vec<(char, &str)> = dropped.iter().map(|(c, r)| (*c, r.as_str())).collect();
    assert!(!faults(&dropped).1.is_empty());
    let reversed: Vec<(char, &str)> = TOPOLOGY
        .iter()
        .map(|&(c, r)| (c, if c == 'リ' { "a1>a2 b3>c1" } else { r }))
        .collect();
    assert!(!faults(&reversed).1.is_empty());
}

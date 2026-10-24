//! alice-sfnt against ttf-parser on valid fonts: every glyph's outline, every advance, and the cmap over the
//! BMP plus the supplementary planes' first code points must agree.
//!
//! Fonts: tests/fonts/* (made by tests/fonts/make_test_fonts.py) always. With ALICE_SFNT_SYSTEM_FONTS=1, also
//! every .ttf / .otf / .ttc under the system font directories and the directories in ALICE_SFNT_FONT_DIRS
//! (`:`-separated): a local run in a release build; nothing derived from those fonts is committed. A face
//! ttf-parser cannot open is skipped; a glyph alice-sfnt reports as Unsupported (CFF seac) is counted and listed.
//! On a system font, a code point only alice-sfnt maps (ttf-parser returns no glyph) is listed for a check with a
//! third reader instead of failing. Any other difference fails, and so does a run that compared no glyph.

fn system_fonts() -> bool {
    std::env::var("ALICE_SFNT_SYSTEM_FONTS").as_deref() == Ok("1")
}

use alice_sfnt::{fonts_in, ErrorKind, Font, OutlineSink};
use std::path::{Path, PathBuf};

#[derive(Default, PartialEq, Debug)]
struct Rec(Vec<(char, [f32; 6])>);

impl OutlineSink for Rec {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push(('M', [x, y, 0.0, 0.0, 0.0, 0.0]));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push(('L', [x, y, 0.0, 0.0, 0.0, 0.0]));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.push(('Q', [x1, y1, x, y, 0.0, 0.0]));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.push(('C', [x1, y1, x2, y2, x, y]));
    }
    fn close(&mut self) {
        self.0.push(('Z', [0.0; 6]));
    }
}

impl ttf_parser::OutlineBuilder for Rec {
    fn move_to(&mut self, x: f32, y: f32) {
        OutlineSink::move_to(self, x, y)
    }
    fn line_to(&mut self, x: f32, y: f32) {
        OutlineSink::line_to(self, x, y)
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        OutlineSink::quad_to(self, x1, y1, x, y)
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        OutlineSink::curve_to(self, x1, y1, x2, y2, x, y)
    }
    fn close(&mut self) {
        OutlineSink::close(self)
    }
}

impl Rec {
    /// ttf-parser does not close the last CFF2 contour; a close before a move or at the end is implied
    fn normalised(mut self) -> Vec<(char, [f32; 6])> {
        self.0.retain(|(k, _)| *k != 'Z');
        self.0
    }
}

fn font_files() -> Vec<PathBuf> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fonts");
    let mut dirs = vec![here];
    if system_fonts() {
        for d in [
            "/System/Library/Fonts",
            "/Library/Fonts",
            "/usr/share/fonts",
            "C:\\Windows\\Fonts",
        ] {
            dirs.push(PathBuf::from(d));
        }
        if let Ok(extra) = std::env::var("ALICE_SFNT_FONT_DIRS") {
            dirs.extend(
                extra
                    .split(':')
                    .filter(|s| !s.is_empty())
                    .map(PathBuf::from),
            );
        }
    }
    let mut out = Vec::new();
    let mut stack: Vec<PathBuf> = dirs.into_iter().filter(|d| d.is_dir()).collect();
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| matches!(x.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc"))
            {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// Fonts whose outlines ttf-parser 0.25.1 cannot draw at all: fontTools' CFF2 without a variation store.
/// tests/oracle.rs pins their outlines against the drawn shapes, so here alice-sfnt only has to read them.
const OUTLINE_REFERENCE_CANNOT_READ: &[&str] = &["cubic2.otf"];

fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

#[derive(Default)]
struct Tally {
    faces: usize,
    glyphs: usize,
    lookups: usize,
    reference_wrong_checked: usize,
    only_ours: Vec<String>,
    skipped_faces: Vec<String>,
    unsupported: Vec<String>,
    differences: Vec<String>,
}

fn compare_face(name: &str, data: &[u8], index: u32, t: &mut Tally) {
    let committed = name.contains("tests/fonts") || name.contains("tests\\fonts");
    let Ok(theirs) = ttf_parser::Face::parse(data, index) else {
        t.skipped_faces.push(format!("{name}#{index}"));
        return;
    };
    let ours = match Font::parse(data, index) {
        Ok(f) => f,
        Err(e) => {
            t.differences.push(format!(
                "{name}#{index}: ttf-parser opens it, alice-sfnt: {e}"
            ));
            return;
        }
    };
    t.faces += 1;
    if ours.units_per_em() != theirs.units_per_em()
        || ours.num_glyphs() != theirs.number_of_glyphs()
    {
        t.differences
            .push(format!("{name}#{index}: units_per_em / glyph count differ"));
        return;
    }
    let mut diffs = 0;
    for gid in 0..ours.num_glyphs() {
        let mut a = Rec::default();
        let mut b = Rec::default();
        let mine = ours.outline(gid, &mut a);
        // the default instance: ttf-parser's glyf table directly (Face::outline_glyph goes through gvar when a
        // font has one, and fails on some variable fonts that fontTools and alice-sfnt read the same way)
        let theirs_drew = match theirs.tables().glyf {
            Some(glyf) => glyf.outline(ttf_parser::GlyphId(gid), &mut b).is_some(),
            None => theirs
                .outline_glyph(ttf_parser::GlyphId(gid), &mut b)
                .is_some(),
        };
        t.glyphs += 1;
        match mine {
            Err(e) if e.kind == ErrorKind::Unsupported => t
                .unsupported
                .push(format!("{name}#{index} glyph {gid}: {e}")),
            Err(e) => {
                if theirs_drew {
                    diffs += 1;
                    if diffs <= 8 {
                        t.differences.push(format!(
                            "{name}#{index} glyph {gid}: alice-sfnt {e}, ttf-parser drew it"
                        ));
                    }
                }
            }
            Ok(_) if !theirs_drew && OUTLINE_REFERENCE_CANNOT_READ.contains(&file_name(name)) => {
                t.reference_wrong_checked += 1;
            }
            Ok(_) => {
                let (a, b) = (a.normalised(), b.normalised());
                // ttf-parser drops an outline whose bounding box is empty; then alice-sfnt's must be empty too
                let same = if theirs_drew {
                    a == b
                } else {
                    a.iter().all(|(k, _)| *k == 'M') || a.is_empty()
                };
                if !same {
                    diffs += 1;
                    if diffs <= 8 {
                        t.differences.push(format!("{name}#{index} glyph {gid}: outlines differ\n  ours   {a:?}\n  theirs {b:?}"));
                    }
                }
            }
        }
        let adv = theirs.glyph_hor_advance(ttf_parser::GlyphId(gid));
        if adv.is_some() && ours.advance(gid).ok() != adv {
            diffs += 1;
            if diffs <= 8 {
                t.differences.push(format!(
                    "{name}#{index} glyph {gid}: advance {:?} vs {adv:?}",
                    ours.advance(gid)
                ));
            }
        }
    }
    for cp in (0u32..0x1_0000)
        .chain(0x1_0000..0x1_0400)
        .chain(0x2_0000..0x2_0400)
    {
        let Some(ch) = char::from_u32(cp) else {
            continue;
        };
        t.lookups += 1;
        let a = ours.glyph_index(cp).ok().flatten();
        // glyph 0 is the missing glyph: ttf-parser returns Some(0) where a format 4 delta wraps to 0
        let b = theirs.glyph_index(ch).map(|g| g.0).filter(|&g| g != 0);
        // a system font (never committed) where only alice-sfnt finds a glyph: listed for a third reader
        if b.is_none() && a.is_some() && !committed {
            t.only_ours.push(format!(
                "{name}#{index} U+{cp:04X}: alice-sfnt {a:?}, ttf-parser none"
            ));
            continue;
        }
        if a != b {
            diffs += 1;
            if diffs <= 8 {
                t.differences
                    .push(format!("{name}#{index} U+{cp:04X}: {a:?} vs {b:?}"));
            }
        }
    }
    if diffs > 8 {
        t.differences
            .push(format!("{name}#{index}: {} more differences", diffs - 8));
    }
}

#[test]
fn outlines_metrics_and_cmap_agree_with_ttf_parser() {
    let files = font_files();
    let mut t = Tally::default();
    for p in &files {
        let Ok(data) = std::fs::read(p) else { continue };
        let name = p.display().to_string();
        let n = fonts_in(&data).unwrap_or(1).min(64);
        for i in 0..n {
            compare_face(&name, &data, i, &mut t);
        }
    }
    eprintln!(
        "differential: {} files, {} faces, {} glyphs, {} lookups, {} checked against a third reader, {} faces skipped (ttf-parser cannot open), {} glyphs unsupported",
        files.len(),
        t.faces,
        t.glyphs,
        t.lookups,
        t.reference_wrong_checked,
        t.skipped_faces.len(),
        t.unsupported.len()
    );
    for u in t.only_ours.iter().take(20) {
        eprintln!("only alice-sfnt maps it (check with a third reader): {u}");
    }
    for u in t.unsupported.iter().take(10) {
        eprintln!("unsupported: {u}");
    }
    assert!(
        t.glyphs > 0 && t.faces >= 4,
        "compared {} faces / {} glyphs: nothing to compare",
        t.faces,
        t.glyphs
    );
    assert!(
        t.differences.is_empty(),
        "{} difference(s):\n{}",
        t.differences.len(),
        t.differences.join("\n")
    );
}

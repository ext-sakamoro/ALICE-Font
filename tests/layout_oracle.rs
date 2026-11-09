//! Oracles of the paragraph layout (src/layout).
//!
//! The kinsoku sets are written here from W3C JLREQ (Requirements for Japanese Text Layout)
//! 3.1.7 "Characters not starting a line" and 3.1.8 "Characters not ending a line" with the
//! class lists of its Appendix A (JIS X 4051), not read from the crate, so a wrong class table
//! in the crate fails them. Widths are checked against the sum of the metrics' advances.
use alice_font::layout::{
    jis, layout, BreakClass, GlyphMetrics, Kinsoku, LayoutError, LayoutParams, LineEnd, MetricsSet,
    Paragraph, ProceduralMetrics, StyleId, StyleSpan,
};
use sha2::{Digest, Sha256};

// oracle: JLREQ 3.1.7 (cl-02 closing brackets, cl-03 hyphens, cl-04 dividing punctuation,
// cl-05 middle dots, cl-06 full stops, cl-07 commas, cl-09 iteration marks, cl-13 postfixed
// abbreviations; cl-10 prolonged sound mark and cl-11 small kana under the strict rule)
const NOT_STARTING: &str =
    "’”）〕］｝〉》」』】⦆〙〗»〟)]}‐〜゠–？！‼⁇⁈⁉・：；。．、，ヽヾゝゞ々〻°′″℃￠％‰㏋ℓ";
const NOT_STARTING_STRICT_ONLY: &str = "ーぁぃぅぇぉっゃゅょゎゕゖァィゥェォッャュョヮヵヶ";
// oracle: JLREQ 3.1.8 (cl-01 opening brackets, cl-12 prefixed abbreviations)
const NOT_ENDING: &str = "‘“（〔［｛〈《「『【⦅〘〖«〝([{￥＄￡＃€№";
// oracle: JLREQ 3.1.10, cl-08: two of the same are not split
const INSEPARABLE: &str = "—…‥";

/// A font for the oracles: ideographs, kana and full-width marks 1000, Latin 500, space 250
/// (upm 1000); the strong style widens Latin to 600. '☃' has no glyph.
struct TestFont {
    latin: i32,
}
impl GlyphMetrics for TestFont {
    fn units_per_em(&self) -> u16 {
        1000
    }
    fn advance(&self, ch: char) -> Option<i32> {
        match ch {
            '☃' => None,
            ' ' => Some(250),
            c if c.is_ascii_graphic() || ('\u{00C0}'..='\u{024F}').contains(&c) => Some(self.latin),
            _ => Some(1000),
        }
    }
}
struct Set {
    regular: TestFont,
    strong: TestFont,
}
const REGULAR: StyleId = StyleId(0);
const STRONG: StyleId = StyleId(1);
impl MetricsSet for Set {
    fn get(&self, s: StyleId) -> Option<&dyn GlyphMetrics> {
        match s {
            REGULAR => Some(&self.regular),
            STRONG => Some(&self.strong),
            _ => None,
        }
    }
}
fn set() -> Set {
    Set {
        regular: TestFont { latin: 500 },
        strong: TestFont { latin: 600 },
    }
}
fn whole(text: &str) -> Vec<StyleSpan> {
    vec![StyleSpan {
        src: 0..text.len() as u32,
        style: REGULAR,
    }]
}
fn params(measure: i32) -> LayoutParams {
    LayoutParams {
        measure,
        line_height: 1500,
        ascent: 1200,
        kinsoku: Kinsoku::Strict,
        hanging_punct: false,
    }
}
fn lay(text: &str, measure: i32) -> Paragraph {
    layout(text, &whole(text), &set(), &params(measure)).unwrap()
}
fn line_text(p: &Paragraph, i: usize) -> String {
    let l = &p.lines[i];
    p.glyphs[l.glyphs.start as usize..l.glyphs.end as usize]
        .iter()
        .map(|g| g.ch)
        .collect()
}

const CORPUS: &[&str] = &[
    "吾輩は猫である。名前はまだ無い。どこで生れたかとんと見当がつかぬ。",
    "「ちょっと待って！」と彼女は言った。……そして、静かにドアを閉めた——二度と開かないように。",
    "コーヒーを飲みながら、ニュースを読む。今日は晴れ、気温は２５℃、湿度は６０％。",
    "価格は￥１，２００（税込）です。詳しくは【お知らせ】をご覧ください。",
    "ALICE-Font lays out text in integer font units, so native and wasm32 agree bit for bit.",
    "日本語と English が混ざった文でも、単語の途中では breaks しない。人々や時々のように、々は行頭に来ない。いすゞ、こゝろ、ほゞ。",
    "ちょっとしたキャッシュの問題で、ショップのページが表示されなかった。",
    "I don’t think it’s Alice’s; we’re told 日本語.英語!と書く人もいる、rock ’n’ roll isn’t dead.",
];

#[test]
fn the_class_table_is_jlreq_appendix_a() {
    use BreakClass as C;
    for (chars, c) in [
        ("「『（【〔［｛〈《‘“", C::OpeningBracket),
        ("」』）】〕］｝〉》’”", C::ClosingBracket),
        ("‐〜゠–", C::Hyphen),
        ("？！‼⁇", C::DividingPunctuation),
        ("・：；", C::MiddleDot),
        ("。．", C::FullStop),
        ("、，", C::Comma),
        ("—…‥", C::Inseparable),
        ("ヽヾゝゞ々〻", C::IterationMark),
        ("ー", C::ProlongedSoundMark),
        (
            "ぁっゃァッャヵヶㇰㇱㇲㇳㇴㇵㇶㇷㇸㇹㇺㇻㇼㇽㇾㇿ",
            C::SmallKana,
        ),
        ("￥＄￡＃€№", C::PrefixedAbbreviation),
        ("°′″℃％‰", C::PostfixedAbbreviation),
        ("\u{3000}", C::IdeographicSpace),
        ("あ漢カA9", C::Other),
    ] {
        for ch in chars.chars() {
            let want = if ch.is_ascii_alphanumeric() {
                C::Western
            } else {
                c
            };
            assert_eq!(jis::class(ch), want, "{ch:?}");
        }
    }
}

/// Every line boundary of the corpus at every measure from 5 em to 20 em keeps the JLREQ rules.
#[test]
fn kinsoku_holds_on_the_corpus_at_every_measure() {
    let mut boundaries = 0;
    for text in CORPUS {
        // 5 em: the longest Latin word of the corpus ("ALICE-Font", 10 x 500) fits, so no line is forced
        for measure in (5000..=20_000).step_by(250) {
            let p = lay(text, measure);
            for i in 1..p.lines.len() {
                let prev = &p.lines[i - 1];
                assert_ne!(
                    prev.end,
                    LineEnd::Forced,
                    "forced break in {text:?} at {measure}"
                );
                let first = p.glyphs[p.lines[i].glyphs.start as usize].ch;
                let mut j = prev.glyphs.end as usize;
                while j > prev.glyphs.start as usize + 1 && p.glyphs[j - 1].ch == ' ' {
                    j -= 1;
                }
                let last = p.glyphs[j - 1].ch;
                assert!(
                    !NOT_STARTING.contains(first),
                    "{first:?} starts a line: {text:?} @ {measure}"
                );
                assert!(
                    !NOT_STARTING_STRICT_ONLY.contains(first),
                    "{first:?} starts a line (strict)"
                );
                assert!(
                    !NOT_ENDING.contains(last),
                    "{last:?} ends a line: {text:?} @ {measure}"
                );
                assert!(
                    !(INSEPARABLE.contains(last) && last == first),
                    "{last:?}{first:?} split"
                );
                let raw_last = p.glyphs[prev.glyphs.end as usize - 1].ch;
                assert!(
                    !(raw_last.is_ascii_alphanumeric() && first.is_ascii_alphanumeric()),
                    "a Latin word is split: {raw_last:?}|{first:?}"
                );
                boundaries += 1;
            }
        }
    }
    assert!(
        boundaries > 1000,
        "the corpus must produce line boundaries ({boundaries})"
    );
}

#[test]
fn loose_kinsoku_lets_small_kana_start_a_line() {
    // "あいうっ" at 3 em: strict pushes "うっ" down, loose breaks before "っ"
    let text = "あいうっえ";
    let strict = lay(text, 3000);
    assert_eq!(line_text(&strict, 0), "あい");
    let mut p = params(3000);
    p.kinsoku = Kinsoku::Loose;
    let loose = layout(text, &whole(text), &set(), &p).unwrap();
    assert_eq!(line_text(&loose, 0), "あいう");
    assert_eq!(line_text(&loose, 1), "っえ");
}

/// A deterministic generator of mixed text, for the width properties.
fn sample(seed: u64, n: usize) -> String {
    const ALPHABET: &[&str] = &[
        "あ", "い", "漢", "字", "カ", "ナ", "ー", "っ", "ャ", "。", "、", "「", "」", "（", "）",
        "…", "…", "—", "！", "？", "・", "々", "％", "￥", "word", "ALICE", "x", " ", " ", "\n",
    ];
    let mut s = seed;
    let mut out = String::new();
    for _ in 0..n {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        out.push_str(ALPHABET[((s >> 33) as usize) % ALPHABET.len()]);
    }
    out
}

#[test]
fn every_line_fits_the_measure_and_its_width_is_its_advances() {
    let fonts = set();
    let mut lines = 0;
    for seed in 0..200u64 {
        let text = sample(seed, 60);
        for measure in [1000, 1250, 2000, 3333, 5000, 8000] {
            for hang in [false, true] {
                let mut prm = params(measure);
                prm.hanging_punct = hang;
                let p = match layout(&text, &whole(&text), &fonts, &prm) {
                    Ok(p) => p,
                    Err(LayoutError::MeasureTooSmall { needed, .. }) => {
                        assert!(needed > measure);
                        continue;
                    }
                    Err(e) => panic!("{e}"),
                };
                let chars = text.chars().filter(|&c| c != '\n').count();
                assert_eq!(
                    p.glyphs.len(),
                    chars,
                    "every character but the newlines is a glyph"
                );
                assert_eq!(p.height, p.lines.len() as i32 * 1500);
                let mut next = 0u32;
                for (i, l) in p.lines.iter().enumerate() {
                    assert!(
                        l.width <= measure,
                        "line {i} is {} > {measure}: {text:?}",
                        l.width
                    );
                    assert!(l.hang == 0 || hang, "a hang without hanging_punct");
                    assert_eq!(l.baseline, i as i32 * 1500 + 1200, "baseline");
                    let g = &p.glyphs[l.glyphs.start as usize..l.glyphs.end as usize];
                    let mut x = 0;
                    for gl in g {
                        assert_eq!(gl.x, x, "pen position");
                        assert_eq!(gl.advance, fonts.regular.advance(gl.ch).unwrap());
                        x += gl.advance;
                    }
                    let trailing: i32 = g
                        .iter()
                        .rev()
                        .take_while(|g| g.ch == ' ')
                        .map(|g| g.advance)
                        .sum();
                    assert_eq!(
                        l.width + l.hang + trailing,
                        x,
                        "width = advances - trailing spaces - hang"
                    );
                    assert!(l.src.start >= next, "lines follow the text");
                    next = l.src.end;
                    lines += 1;
                }
            }
        }
    }
    assert!(lines > 5000, "the property must see many lines ({lines})");
}

#[test]
fn a_full_stop_hangs_only_when_asked() {
    let text = "あいう。えお";
    let off = lay(text, 3000);
    assert_eq!(
        line_text(&off, 0),
        "あい",
        "the mark cannot start a line, so う goes down with it"
    );
    assert_eq!(off.lines[0].width, 2000);
    let mut p = params(3000);
    p.hanging_punct = true;
    let on = layout(text, &whole(text), &set(), &p).unwrap();
    assert_eq!(line_text(&on, 0), "あいう。");
    assert_eq!((on.lines[0].width, on.lines[0].hang), (3000, 1000));
}

// A mark after a space does not hang: the space would then sit inside the line's content and
// push it past the measure ("あい " is 2250 > 2100).
#[test]
fn a_mark_after_a_space_does_not_hang() {
    let text = "あい 。う";
    let mut p = params(2100);
    p.hanging_punct = true;
    let out = layout(text, &whole(text), &set(), &p).unwrap();
    for l in &out.lines {
        assert!(l.width <= 2100, "{l:?}");
    }
    assert_eq!(out.lines[0].hang, 0);
}

#[test]
fn latin_breaks_after_spaces_and_trailing_spaces_take_no_width() {
    let p = lay("hello world foo", 5500);
    assert_eq!(line_text(&p, 0), "hello world ");
    assert_eq!(p.lines[0].width, 5250);
    assert_eq!(line_text(&p, 1), "foo");
    assert_eq!(p.lines[1].end, LineEnd::End);
    let hard = lay("ab\ncd", 9000);
    assert_eq!((hard.lines.len(), hard.lines[0].end), (2, LineEnd::Hard));
    assert_eq!(hard.lines[1].src, 3..5);
}

// A typographic quote inside a Latin word (don’t, it’s, ‘quoted’) joins it: no break on either side
// of it between Western characters. At 3 em, "don’t" (5 x 500 = 2500) fits on the second line.
#[test]
fn a_quote_inside_a_latin_word_does_not_split_it() {
    for text in [
        "I don’t know",
        "it’s fine",
        "say ‘yes’ now",
        "a “b” c",
        "we can’t and won’t, they’ll say we’re late",
        "Alice’s and James’ books",
        "the 1990’s and o’clock",
        "rock ’n’ roll isn’t dead",
        "shouldn’t’ve",
    ] {
        for measure in (1500..=6000).step_by(250) {
            let Ok(p) = layout(text, &whole(text), &set(), &params(measure)) else {
                continue;
            };
            for i in 1..p.lines.len() {
                if p.lines[i - 1].end != LineEnd::Soft {
                    continue;
                }
                let a = p.glyphs[p.lines[i - 1].glyphs.end as usize - 1].ch;
                let b = p.glyphs[p.lines[i].glyphs.start as usize].ch;
                let latin = |c: char| c.is_ascii_alphanumeric();
                let quote = |c: char| "‘’“”".contains(c);
                assert!(
                    !((latin(a) && quote(b)) || (quote(a) && latin(b))),
                    "{text:?} at {measure} breaks {a:?}|{b:?}"
                );
            }
        }
    }
}

// oracle: UAX #14 classes IS (. , : ;) and EX (! ?) take no break before them, so Latin full
// stops and the like never start a line, after a Latin or a Japanese character alike
#[test]
fn latin_punctuation_never_starts_a_line() {
    for text in [
        "日本語.英語",
        "漢字,かな",
        "本当!です",
        "何?何",
        "時:分",
        "ab;cd",
    ] {
        for measure in (1000..=4000).step_by(250) {
            let Ok(p) = layout(text, &whole(text), &set(), &params(measure)) else {
                continue;
            };
            // a Forced line end (nothing fits) breaks without the rules; the others are checked
            for w in p.lines.windows(2) {
                if w[0].end != LineEnd::Soft {
                    continue;
                }
                let first = p.glyphs[w[1].glyphs.start as usize].ch;
                assert!(
                    !".,:;!?".contains(first),
                    "{text:?} at {measure}: line starts with {first:?}"
                );
            }
        }
    }
}

#[test]
fn a_word_longer_than_the_measure_is_forced() {
    let p = lay("abcdefgh", 2000);
    assert_eq!(line_text(&p, 0), "abcd");
    assert_eq!(p.lines[0].end, LineEnd::Forced);
}

#[test]
fn strong_spans_use_their_own_advances() {
    // "ab cd": regular 500 → "ab cd" fits 2250; strong "cd" (600 each) makes it 2450
    let text = "ab cd";
    let spans = vec![
        StyleSpan {
            src: 0..3,
            style: REGULAR,
        },
        StyleSpan {
            src: 3..5,
            style: STRONG,
        },
    ];
    let p = layout(text, &spans, &set(), &params(2300)).unwrap();
    assert_eq!(p.lines.len(), 2);
    assert_eq!(p.glyphs[3].style, STRONG);
    assert_eq!(p.glyphs[3].advance, 600);
    assert_eq!(
        layout(text, &whole(text), &set(), &params(2300))
            .unwrap()
            .lines
            .len(),
        1
    );
}

#[test]
fn spans_must_cover_the_text_exactly() {
    let text = "あいう"; // 9 bytes
    let f = set();
    let bad = |spans: Vec<StyleSpan>| {
        matches!(
            layout(text, &spans, &f, &params(9000)),
            Err(LayoutError::BadParams(_))
        )
    };
    let s = |a: u32, b: u32, st: StyleId| StyleSpan {
        src: a..b,
        style: st,
    };
    assert!(bad(vec![]), "no span");
    assert!(bad(vec![s(0, 3, REGULAR), s(6, 9, REGULAR)]), "gap");
    assert!(bad(vec![s(0, 6, REGULAR), s(3, 9, REGULAR)]), "overlap");
    assert!(bad(vec![s(3, 9, REGULAR), s(0, 3, REGULAR)]), "unsorted");
    assert!(bad(vec![s(0, 12, REGULAR)]), "past the end");
    assert!(
        bad(vec![s(0, 4, REGULAR), s(4, 9, REGULAR)]),
        "off a character boundary"
    );
    assert!(bad(vec![s(0, 9, StyleId(7))]), "a style the set lacks");
    assert!(
        bad(vec![s(0, 0, REGULAR), s(0, 9, REGULAR)]),
        "an empty span"
    );
    assert!(!bad(vec![s(0, 3, REGULAR), s(3, 9, STRONG)]));
    for (m, h, a) in [
        (0, 1500, 0),
        (-1, 1500, 0),
        (1000, 0, 0),
        (1000, 1500, -1),
        (1000, 1500, 1501),
    ] {
        let p = LayoutParams {
            measure: m,
            line_height: h,
            ascent: a,
            ..params(1)
        };
        assert!(matches!(
            layout(text, &whole(text), &f, &p),
            Err(LayoutError::BadParams(_))
        ));
    }
}

#[test]
fn an_empty_text_is_an_empty_paragraph() {
    let p = layout("", &[], &set(), &params(1000)).unwrap();
    assert_eq!((p.glyphs.len(), p.lines.len(), p.height), (0, 0, 0));
}

#[test]
fn refusals_name_the_glyph() {
    let text = "あ☃";
    assert_eq!(
        layout(text, &whole(text), &set(), &params(5000)),
        Err(LayoutError::UnknownGlyph { ch: '☃', src: 3 })
    );
    assert_eq!(
        layout("漢", &whole("漢"), &set(), &params(999)),
        Err(LayoutError::MeasureTooSmall {
            needed: 1000,
            src: 0
        })
    );
}

#[test]
fn procedural_advances_are_the_skeleton_em_rounded_once() {
    let m = ProceduralMetrics {
        params: alice_font::param::MetaFontParams::sans_regular(),
        units_per_em: 1000,
    };
    for ch in ['A', 'g', 'ア', 'あ', '永', '\u{3000}'] {
        let s = alice_font::glyph::dispatcher::skeleton(ch, &m.params).unwrap();
        // oracle: the closed form, round(advance_em * 1000) ties away from zero
        let want = (f64::from(s.advance) * 1000.0).round() as i32;
        assert_eq!(m.advance(ch), Some(want), "{ch:?}");
    }
    assert_eq!(m.advance('☃'), None);
}

/// The bits of a fixed corpus: the same on every target. CI runs this test natively and on
/// wasm32-wasip1 (wasmtime); both must match the recorded digest.
#[test]
fn the_corpus_layout_is_the_recorded_bits() {
    let mut h = Sha256::new();
    for text in CORPUS {
        for measure in [3000, 7000, 12_000] {
            let p = lay(text, measure);
            for g in &p.glyphs {
                h.update(u32::from(g.ch).to_le_bytes());
                h.update(g.src.to_le_bytes());
                h.update(g.x.to_le_bytes());
                h.update(g.advance.to_le_bytes());
            }
            for l in &p.lines {
                for v in [l.glyphs.start, l.glyphs.end, l.src.start, l.src.end] {
                    h.update(v.to_le_bytes());
                }
                for v in [l.width, l.hang, l.baseline] {
                    h.update(v.to_le_bytes());
                }
                h.update([l.end as u8]);
            }
        }
    }
    let got: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        got, "f932be5b592c021efc7266266c814cd623c17f74f8d263b0d69ff57ff837bfee",
        "re-record only for an intended change"
    );
}

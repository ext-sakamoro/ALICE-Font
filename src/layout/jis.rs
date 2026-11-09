//! Character classes of JIS X 4051 (as listed in W3C "Requirements for Japanese Text Layout",
//! JLREQ, Appendix A: cl-01 ... cl-30) and the line-breaking rules this layout applies.
//!
//! Only the classes the rules below read are told apart; every other character is `Other`
//! (ideographs, kana, Latin letters and digits fall in their own groups because the rules
//! treat them differently).

/// The classes of JLREQ Appendix A that line breaking reads. The discriminant is the JLREQ class
/// number (cl-NN); the three groups outside its list take 0, 100 and 101. `LAYOUT_SEMANTICS_ID`
/// folds a digest of the class of every code point under these numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BreakClass {
    /// cl-01 opening brackets: never at the end of a line
    OpeningBracket = 1,
    /// cl-02 closing brackets: never at the start of a line
    ClosingBracket = 2,
    /// cl-03 hyphens: never at the start of a line
    Hyphen = 3,
    /// cl-04 dividing punctuation (？ ！): never at the start of a line
    DividingPunctuation = 4,
    /// cl-05 middle dots (・ ： ；): never at the start of a line
    MiddleDot = 5,
    /// cl-06 full stops (。 ．): never at the start of a line; may hang
    FullStop = 6,
    /// cl-07 commas (、 ，): never at the start of a line; may hang
    Comma = 7,
    /// cl-08 inseparable characters (— … ‥): two of the same are not split
    Inseparable = 8,
    /// cl-09 iteration marks (ヽ ヾ ゝ ゞ 々 〻): never at the start of a line
    IterationMark = 9,
    /// cl-10 prolonged sound mark (ー): not at the start of a line under `Kinsoku::Strict`
    ProlongedSoundMark = 10,
    /// cl-11 small kana (ぁ ッ ...): not at the start of a line under `Kinsoku::Strict`
    SmallKana = 11,
    /// cl-12 prefixed abbreviations (￥ ＄ ...): never at the end of a line
    PrefixedAbbreviation = 12,
    /// cl-13 postfixed abbreviations (％ ℃ ...): never at the start of a line
    PostfixedAbbreviation = 13,
    /// cl-14 full-width ideographic space (U+3000)
    IdeographicSpace = 14,
    /// A break opportunity after it (U+0020 and the other spaces Latin text breaks at)
    Space = 100,
    /// Latin letters, digits and the punctuation of Latin text: no break inside a run of them
    Western = 101,
    /// Ideographs, kana and everything else: a break is allowed on either side
    Other = 0,
}

const OPENING: &[char] = &[
    '‘', '“', '（', '〔', '［', '｛', '〈', '《', '「', '『', '【', '⦅', '〘', '〖', '«', '〝',
    '(', '[', '{',
];
const CLOSING: &[char] = &[
    '’', '”', '）', '〕', '］', '｝', '〉', '》', '」', '』', '】', '⦆', '〙', '〗', '»', '〟',
    ')', ']', '}',
];
const HYPHEN: &[char] = &['‐', '〜', '゠', '–'];
const DIVIDING: &[char] = &['？', '！', '‼', '⁇', '⁈', '⁉'];
const MIDDLE_DOT: &[char] = &['・', '：', '；'];
const FULL_STOP: &[char] = &['。', '．'];
const COMMA: &[char] = &['、', '，'];
const INSEPARABLE: &[char] = &['—', '…', '‥', '〳', '〴', '〵'];
const ITERATION: &[char] = &['ヽ', 'ヾ', 'ゝ', 'ゞ', '々', '〻'];
const PROLONGED: &[char] = &['ー'];
const SMALL_KANA: &[char] = &[
    'ぁ', 'ぃ', 'ぅ', 'ぇ', 'ぉ', 'っ', 'ゃ', 'ゅ', 'ょ', 'ゎ', 'ゕ', 'ゖ', 'ァ', 'ィ', 'ゥ', 'ェ',
    'ォ', 'ッ', 'ャ', 'ュ', 'ョ', 'ヮ', 'ヵ', 'ヶ', 'ㇰ', 'ㇱ', 'ㇲ', 'ㇳ', 'ㇴ', 'ㇵ', 'ㇶ', 'ㇷ',
    'ㇸ', 'ㇹ', 'ㇺ', 'ㇻ', 'ㇼ', 'ㇽ', 'ㇾ', 'ㇿ',
];
const PREFIXED: &[char] = &['￥', '＄', '￡', '＃', '€', '№'];
const POSTFIXED: &[char] = &['°', '′', '″', '℃', '￠', '％', '‰', '㏋', 'ℓ'];

/// The class of `ch`.
#[must_use]
pub fn class(ch: char) -> BreakClass {
    use BreakClass as C;
    let tables: [(&[char], BreakClass); 13] = [
        (OPENING, C::OpeningBracket),
        (CLOSING, C::ClosingBracket),
        (HYPHEN, C::Hyphen),
        (DIVIDING, C::DividingPunctuation),
        (MIDDLE_DOT, C::MiddleDot),
        (FULL_STOP, C::FullStop),
        (COMMA, C::Comma),
        (INSEPARABLE, C::Inseparable),
        (ITERATION, C::IterationMark),
        (PROLONGED, C::ProlongedSoundMark),
        (SMALL_KANA, C::SmallKana),
        (PREFIXED, C::PrefixedAbbreviation),
        (POSTFIXED, C::PostfixedAbbreviation),
    ];
    for (set, c) in tables {
        if set.contains(&ch) {
            return c;
        }
    }
    match ch {
        '\u{3000}' => C::IdeographicSpace,
        ' ' | '\t' | '\u{2002}' | '\u{2003}' | '\u{2009}' => C::Space,
        c if c.is_ascii_alphanumeric() || c.is_ascii_punctuation() => C::Western,
        c if ('\u{00C0}'..='\u{024F}').contains(&c) => C::Western,
        _ => C::Other,
    }
}

/// How strictly the start of a line is guarded (the two tiers of JIS X 4051 / JLREQ 3.1.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kinsoku {
    /// Also keeps small kana and the prolonged sound mark off the start of a line
    Strict,
    /// Allows small kana and the prolonged sound mark at the start of a line
    Loose,
}

/// Whether a character of class `c` may begin a line.
#[must_use]
pub fn may_start_line(c: BreakClass, k: Kinsoku) -> bool {
    use BreakClass as C;
    match c {
        C::ClosingBracket
        | C::Hyphen
        | C::DividingPunctuation
        | C::MiddleDot
        | C::FullStop
        | C::Comma
        | C::IterationMark
        | C::PostfixedAbbreviation => false,
        C::ProlongedSoundMark | C::SmallKana => k == Kinsoku::Loose,
        _ => true,
    }
}

/// Whether a character of class `c` may end a line.
#[must_use]
pub fn may_end_line(c: BreakClass) -> bool {
    !matches!(
        c,
        BreakClass::OpeningBracket | BreakClass::PrefixedAbbreviation
    )
}

/// Whether a line may break between `a` and `b` (the characters themselves, then their classes).
#[must_use]
pub fn break_allowed(a: char, b: char, k: Kinsoku) -> bool {
    let (ca, cb) = (class(a), class(b));
    if !may_end_line(ca) || !may_start_line(cb, k) {
        return false;
    }
    // two of the same inseparable character (—— ……) stay together
    if ca == BreakClass::Inseparable && a == b {
        return false;
    }
    // JLREQ's classes order Japanese text; inside Latin text a quote joins the word it touches
    // (don’t, it’s, ‘yes’): no break between a quote and a Western character, either way
    // (UAX #14 treats ’ there as part of the word)
    let quote = |c: char| matches!(c, '‘' | '’' | '“' | '”');
    if (ca == BreakClass::Western && quote(b)) || (quote(a) && cb == BreakClass::Western) {
        return false;
    }
    // UAX #14 classes IS (. , : ;) and EX (! ?): no break before them, after any character
    if matches!(b, '.' | ',' | ':' | ';' | '!' | '?') {
        return false;
    }
    match (ca, cb) {
        // a space is followed by a break opportunity, never preceded by one
        (_, BreakClass::Space) => false,
        (BreakClass::Space, _) => true,
        // a run of Latin letters, digits and their punctuation is one word
        (BreakClass::Western, BreakClass::Western) => false,
        _ => true,
    }
}

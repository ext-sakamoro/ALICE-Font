//! IDS (Ideographic Description Sequence) パーサとツリー表現。
//!
//! 漢字の構造を記述する Unicode 標準形式。例: 明 = "⿰日月"。
//! 詳細は `docs/CJK_KANJI_SPEC.md` 参照。
//!
//! License: MIT
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::str::Chars;

use crate::cjk::layout::CompositionLayout;

/// IDS ツリー — 漢字の合成構造。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ids {
    /// 終端 — 単一文字 (部首 or 既定義漢字)。
    Leaf(char),
    /// 二項合成 (`⿰⿱⿴⿵⿶⿷⿸⿹⿺⿻`)。
    Binary {
        layout: CompositionLayout,
        first: Box<Self>,
        second: Box<Self>,
    },
    /// 三項合成 (`⿲⿳`)。
    Ternary {
        layout: CompositionLayout,
        first: Box<Self>,
        second: Box<Self>,
        third: Box<Self>,
    },
}

/// IDS パースエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdsParseError {
    /// 空入力。
    Empty,
    /// 不完全な IDS — オペレータの引数が足りない。
    Truncated,
    /// 認識できない文字。
    UnknownChar(char),
    /// パース後に余分な文字が残った。
    TrailingChars,
}

/// IDS 文字列をパースして [`Ids`] ツリーを返す。
///
/// 例: `parse("⿰日月")` → `Binary { layout: LeftRight, first: Leaf('日'), second: Leaf('月') }`
///
/// # Errors
/// 入力が空、不完全、または認識できない文字を含む場合。
pub fn parse(input: &str) -> Result<Ids, IdsParseError> {
    if input.is_empty() {
        return Err(IdsParseError::Empty);
    }
    let mut chars = input.chars();
    let tree = parse_one(&mut chars)?;
    if chars.next().is_some() {
        return Err(IdsParseError::TrailingChars);
    }
    Ok(tree)
}

fn parse_one(chars: &mut Chars<'_>) -> Result<Ids, IdsParseError> {
    let c = chars.next().ok_or(IdsParseError::Truncated)?;
    if let Some(layout) = CompositionLayout::from_ids_char(c) {
        match layout.arity() {
            2 => {
                let first = parse_one(chars)?;
                let second = parse_one(chars)?;
                Ok(Ids::Binary {
                    layout,
                    first: Box::new(first),
                    second: Box::new(second),
                })
            }
            3 => {
                let first = parse_one(chars)?;
                let second = parse_one(chars)?;
                let third = parse_one(chars)?;
                Ok(Ids::Ternary {
                    layout,
                    first: Box::new(first),
                    second: Box::new(second),
                    third: Box::new(third),
                })
            }
            _ => unreachable!("arity must be 2 or 3"),
        }
    } else if is_terminal(c) {
        Ok(Ids::Leaf(c))
    } else {
        Err(IdsParseError::UnknownChar(c))
    }
}

/// 終端文字判定 (IDS 演算子・制御文字以外)。
fn is_terminal(c: char) -> bool {
    !c.is_control() && CompositionLayout::from_ids_char(c).is_none()
}

impl Ids {
    /// このツリーに含まれる終端文字を全て収集する (重複あり)。
    #[must_use]
    pub fn leaves(&self) -> Vec<char> {
        let mut out = Vec::new();
        self.collect_leaves(&mut out);
        out
    }

    fn collect_leaves(&self, out: &mut Vec<char>) {
        match self {
            Self::Leaf(c) => out.push(*c),
            Self::Binary { first, second, .. } => {
                first.collect_leaves(out);
                second.collect_leaves(out);
            }
            Self::Ternary {
                first,
                second,
                third,
                ..
            } => {
                first.collect_leaves(out);
                second.collect_leaves(out);
                third.collect_leaves(out);
            }
        }
    }

    /// ツリーの最大ネスト深さ。`Leaf` は 0、二項合成は `1 + max(children)` を返す。
    #[must_use]
    pub fn depth(&self) -> usize {
        match self {
            Self::Leaf(_) => 0,
            Self::Binary { first, second, .. } => 1 + first.depth().max(second.depth()),
            Self::Ternary {
                first,
                second,
                third,
                ..
            } => 1 + first.depth().max(second.depth()).max(third.depth()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn parse_single_leaf() {
        assert_eq!(parse("日").unwrap(), Ids::Leaf('日'));
    }

    #[test]
    fn parse_simple_left_right() {
        // 明 = ⿰日月
        let tree = parse("⿰日月").unwrap();
        match tree {
            Ids::Binary {
                layout,
                first,
                second,
            } => {
                assert_eq!(layout, CompositionLayout::LeftRight);
                assert_eq!(*first, Ids::Leaf('日'));
                assert_eq!(*second, Ids::Leaf('月'));
            }
            _ => panic!("expected Binary"),
        }
    }

    #[test]
    fn parse_top_bottom() {
        let tree = parse("⿱木目").unwrap();
        assert_eq!(tree.depth(), 1);
        assert_eq!(tree.leaves(), vec!['木', '目']);
    }

    #[test]
    fn parse_nested() {
        // 森 = ⿱木⿰木木 (上に木、下に左右木木)
        let tree = parse("⿱木⿰木木").unwrap();
        assert_eq!(tree.depth(), 2);
        assert_eq!(tree.leaves(), vec!['木', '木', '木']);
    }

    #[test]
    fn parse_three_levels() {
        // 章 = ⿳立日十
        let tree = parse("⿳立日十").unwrap();
        match tree {
            Ids::Ternary {
                layout,
                first,
                second,
                third,
            } => {
                assert_eq!(layout, CompositionLayout::TopMidBottom);
                assert_eq!(*first, Ids::Leaf('立'));
                assert_eq!(*second, Ids::Leaf('日'));
                assert_eq!(*third, Ids::Leaf('十'));
            }
            _ => panic!("expected Ternary"),
        }
    }

    #[test]
    fn parse_empty_fails() {
        assert_eq!(parse(""), Err(IdsParseError::Empty));
    }

    #[test]
    fn parse_truncated_fails() {
        // ⿰ requires two children but only one is given.
        assert_eq!(parse("⿰日"), Err(IdsParseError::Truncated));
    }

    #[test]
    fn parse_trailing_chars_fails() {
        assert_eq!(parse("日月"), Err(IdsParseError::TrailingChars));
    }

    #[test]
    fn parse_unknown_char_fails() {
        // Control character (e.g. NUL) is not a valid terminal.
        let result = parse("\u{0001}");
        assert!(matches!(result, Err(IdsParseError::UnknownChar(_))));
    }

    #[test]
    fn enclosure_layouts_parse() {
        for c in ['⿴', '⿵', '⿶', '⿷', '⿸', '⿹', '⿺', '⿻'] {
            let mut buf = alloc::string::String::new();
            buf.push(c);
            buf.push('国');
            buf.push('玉');
            let tree = parse(&buf).unwrap();
            assert_eq!(tree.depth(), 1);
        }
    }

    #[test]
    fn leaves_for_deep_tree() {
        // 警 ≈ ⿱⿰苟攵言 (depth 2)
        let tree = parse("⿱⿰苟攵言").unwrap();
        assert_eq!(tree.depth(), 2);
        assert_eq!(tree.leaves(), vec!['苟', '攵', '言']);
    }
}

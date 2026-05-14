//! 漢字合成レイアウト — IDS (Ideographic Description Sequence) の 12 オペレータ
//!
//! Unicode の Ideographic Description Characters (U+2FF0 - U+2FFB) に対応。
//!
//! License: MIT
//! Author: Moroya Sakamoto

/// IDS の合成オペレータ。`⿰⿱⿲⿳⿴⿵⿶⿷⿸⿹⿺⿻` に対応。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompositionLayout {
    /// 左右配置 (`⿰`)。左半分 + 右半分。例: 明 = ⿰日月
    LeftRight = 0,
    /// 上下配置 (`⿱`)。上半分 + 下半分。例: 男 = ⿱田力
    TopBottom = 1,
    /// 左中右 (`⿲`)。三等分配置。例: 街 (の一部)
    LeftMidRight = 2,
    /// 上中下 (`⿳`)。三段配置。例: 章 = ⿳立日十
    TopMidBottom = 3,
    /// 全囲み (`⿴`)。外枠 + 中身。例: 国 = ⿴囗玉
    Enclosure = 4,
    /// 上囲み (`⿵`)。⊓型 + 中身。例: 同 = ⿵冂一口
    TopSurround = 5,
    /// 下囲み (`⿶`)。⊔型 + 中身。例: 凶 = ⿶凵乂
    BottomSurround = 6,
    /// 左囲み (`⿷`)。⊏型 + 中身。例: 区 = ⿷匚乂
    LeftSurround = 7,
    /// 左上囲み (`⿸`)。Γ型 + 中身。例: 病 = ⿸疒丙
    TopLeftSurround = 8,
    /// 右上囲み (`⿹`)。⌐型 + 中身。例: 句 = ⿹勹口
    TopRightSurround = 9,
    /// 左下囲み (`⿺`)。⌊型 + 中身。例: 道 = ⿺辶首
    BottomLeftSurround = 10,
    /// 重ね (`⿻`)。要素同士の重ね合わせ。
    Overlay = 11,
}

impl CompositionLayout {
    /// IDS 文字 (`U+2FF0`〜`U+2FFB`) からレイアウトを引く。
    #[must_use]
    pub const fn from_ids_char(c: char) -> Option<Self> {
        match c {
            '\u{2FF0}' => Some(Self::LeftRight),
            '\u{2FF1}' => Some(Self::TopBottom),
            '\u{2FF2}' => Some(Self::LeftMidRight),
            '\u{2FF3}' => Some(Self::TopMidBottom),
            '\u{2FF4}' => Some(Self::Enclosure),
            '\u{2FF5}' => Some(Self::TopSurround),
            '\u{2FF6}' => Some(Self::BottomSurround),
            '\u{2FF7}' => Some(Self::LeftSurround),
            '\u{2FF8}' => Some(Self::TopLeftSurround),
            '\u{2FF9}' => Some(Self::TopRightSurround),
            '\u{2FFA}' => Some(Self::BottomLeftSurround),
            '\u{2FFB}' => Some(Self::Overlay),
            _ => None,
        }
    }

    /// 対応する IDS 文字。
    #[must_use]
    pub const fn to_ids_char(self) -> char {
        match self {
            Self::LeftRight => '\u{2FF0}',
            Self::TopBottom => '\u{2FF1}',
            Self::LeftMidRight => '\u{2FF2}',
            Self::TopMidBottom => '\u{2FF3}',
            Self::Enclosure => '\u{2FF4}',
            Self::TopSurround => '\u{2FF5}',
            Self::BottomSurround => '\u{2FF6}',
            Self::LeftSurround => '\u{2FF7}',
            Self::TopLeftSurround => '\u{2FF8}',
            Self::TopRightSurround => '\u{2FF9}',
            Self::BottomLeftSurround => '\u{2FFA}',
            Self::Overlay => '\u{2FFB}',
        }
    }

    /// このレイアウトが必要とする子要素数 (`⿲⿳` は 3、それ以外は 2)。
    #[must_use]
    pub const fn arity(self) -> usize {
        match self {
            Self::LeftMidRight | Self::TopMidBottom => 3,
            _ => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_twelve_variants_round_trip() {
        let all = [
            CompositionLayout::LeftRight,
            CompositionLayout::TopBottom,
            CompositionLayout::LeftMidRight,
            CompositionLayout::TopMidBottom,
            CompositionLayout::Enclosure,
            CompositionLayout::TopSurround,
            CompositionLayout::BottomSurround,
            CompositionLayout::LeftSurround,
            CompositionLayout::TopLeftSurround,
            CompositionLayout::TopRightSurround,
            CompositionLayout::BottomLeftSurround,
            CompositionLayout::Overlay,
        ];
        for layout in all {
            let ch = layout.to_ids_char();
            assert_eq!(CompositionLayout::from_ids_char(ch), Some(layout));
        }
    }

    #[test]
    fn arity_two_for_binary_layouts() {
        assert_eq!(CompositionLayout::LeftRight.arity(), 2);
        assert_eq!(CompositionLayout::Enclosure.arity(), 2);
    }

    #[test]
    fn arity_three_for_ternary_layouts() {
        assert_eq!(CompositionLayout::LeftMidRight.arity(), 3);
        assert_eq!(CompositionLayout::TopMidBottom.arity(), 3);
    }

    #[test]
    fn from_ids_char_rejects_non_ids() {
        assert!(CompositionLayout::from_ids_char('A').is_none());
        assert!(CompositionLayout::from_ids_char('日').is_none());
    }
}

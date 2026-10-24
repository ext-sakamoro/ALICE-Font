//! Bidirectional テキスト (RTL)
//!
//! Unicode UAX#9 簡易実装。
//! アラビア語・ヘブライ語の右から左 (RTL) テキスト処理。

use alloc::vec::Vec;

/// Bidi 文字クラス (簡易版)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BidiClass {
    /// 左から右 (ラテン文字、CJK)。
    L,
    /// 右から左 (アラビア語)。
    AL,
    /// 右から左 (ヘブライ語)。
    R,
    /// アラビア数字。
    AN,
    /// ヨーロッパ数字。
    EN,
    /// ヨーロッパ数字セパレータ。
    ES,
    /// 空白。
    WS,
    /// その他 (ニュートラル)。
    ON,
}

/// 文字の Bidi クラスを判定。
#[must_use]
pub const fn classify(c: char) -> BidiClass {
    let cp = c as u32;
    match cp {
        // アラビア-インド数字 (アラビア文字ブロック内だが数字として分類)
        0x0660..=0x0669 | 0x06F0..=0x06F9 => BidiClass::AN,
        // アラビア文字
        0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF => {
            BidiClass::AL
        }
        // ヘブライ文字
        0x0590..=0x05FF | 0xFB1D..=0xFB4F => BidiClass::R,
        // ヨーロッパ数字
        0x0030..=0x0039 => BidiClass::EN,
        // 空白
        0x0020 | 0x0009 | 0x000A | 0x000D => BidiClass::WS,
        // ラテン文字・CJK等
        0x0041..=0x005A | 0x0061..=0x007A | 0x00C0..=0x024F | 0x4E00..=0x9FFF | 0x3040..=0x30FF => {
            BidiClass::L
        }
        // その他
        _ => BidiClass::ON,
    }
}

/// Bidi 埋め込みレベル。
#[derive(Debug, Clone)]
pub struct BidiLevel {
    /// 文字。
    pub ch: char,
    /// 埋め込みレベル (偶数=LTR、奇数=RTL)。
    pub level: u8,
    /// Bidi クラス。
    pub class: BidiClass,
}

/// 段落方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParagraphDirection {
    /// 左から右。
    Ltr,
    /// 右から左。
    Rtl,
}

/// Bidi リゾルバ。
#[derive(Debug)]
pub struct BidiResolver {
    /// 基本方向。
    pub base_direction: ParagraphDirection,
}

impl BidiResolver {
    /// 新しい Bidi リゾルバを作成。
    #[must_use]
    pub const fn new(base_direction: ParagraphDirection) -> Self {
        Self { base_direction }
    }

    /// 段落方向を自動検出。
    #[must_use]
    pub fn auto_detect(text: &str) -> ParagraphDirection {
        for c in text.chars() {
            match classify(c) {
                BidiClass::L => return ParagraphDirection::Ltr,
                BidiClass::R | BidiClass::AL => return ParagraphDirection::Rtl,
                _ => {}
            }
        }
        ParagraphDirection::Ltr
    }

    /// 埋め込みレベルを計算。
    #[must_use]
    pub fn resolve_levels(&self, text: &str) -> Vec<BidiLevel> {
        let base_level: u8 = match self.base_direction {
            ParagraphDirection::Ltr => 0,
            ParagraphDirection::Rtl => 1,
        };

        text.chars()
            .map(|ch| {
                let class = classify(ch);
                let level = match class {
                    BidiClass::L | BidiClass::EN => {
                        if base_level % 2 == 1 {
                            base_level + 1
                        } else {
                            base_level
                        }
                    }
                    BidiClass::R | BidiClass::AL | BidiClass::AN => {
                        if base_level.is_multiple_of(2) {
                            base_level + 1
                        } else {
                            base_level
                        }
                    }
                    _ => base_level,
                };
                BidiLevel { ch, level, class }
            })
            .collect()
    }

    /// 表示順序にリオーダー。
    #[must_use]
    pub fn reorder(&self, levels: &[BidiLevel]) -> Vec<char> {
        if levels.is_empty() {
            return Vec::new();
        }

        let max_level = levels.iter().map(|l| l.level).max().unwrap_or(0);
        let mut indices: Vec<usize> = (0..levels.len()).collect();

        // 最高レベルから基本レベルまで、各レベルの連続ランを反転
        let base_level: u8 = match self.base_direction {
            ParagraphDirection::Ltr => 0,
            ParagraphDirection::Rtl => 1,
        };

        for level in ((base_level + 1)..=max_level).rev() {
            let mut i = 0;
            while i < indices.len() {
                if levels[indices[i]].level >= level {
                    let start = i;
                    while i < indices.len() && levels[indices[i]].level >= level {
                        i += 1;
                    }
                    indices[start..i].reverse();
                } else {
                    i += 1;
                }
            }
        }

        indices.iter().map(|&i| levels[i].ch).collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn classify_latin() {
        assert_eq!(classify('A'), BidiClass::L);
        assert_eq!(classify('z'), BidiClass::L);
    }

    #[test]
    fn classify_arabic() {
        assert_eq!(classify('\u{0627}'), BidiClass::AL); // أ
    }

    #[test]
    fn classify_hebrew() {
        assert_eq!(classify('\u{05D0}'), BidiClass::R); // א
    }

    #[test]
    fn classify_digit() {
        assert_eq!(classify('5'), BidiClass::EN);
    }

    #[test]
    fn classify_space() {
        assert_eq!(classify(' '), BidiClass::WS);
    }

    #[test]
    fn auto_detect_ltr() {
        assert_eq!(BidiResolver::auto_detect("Hello"), ParagraphDirection::Ltr);
    }

    #[test]
    fn auto_detect_rtl() {
        assert_eq!(
            BidiResolver::auto_detect("\u{05D0}\u{05D1}\u{05D2}"),
            ParagraphDirection::Rtl
        );
    }

    #[test]
    fn auto_detect_empty() {
        assert_eq!(BidiResolver::auto_detect(""), ParagraphDirection::Ltr);
    }

    #[test]
    fn resolve_ltr_text() {
        let resolver = BidiResolver::new(ParagraphDirection::Ltr);
        let levels = resolver.resolve_levels("ABC");
        assert!(levels.iter().all(|l| l.level == 0));
    }

    #[test]
    fn resolve_rtl_in_ltr() {
        let resolver = BidiResolver::new(ParagraphDirection::Ltr);
        let levels = resolver.resolve_levels("A\u{05D0}B");
        assert_eq!(levels[0].level, 0); // A
        assert_eq!(levels[1].level, 1); // Hebrew
        assert_eq!(levels[2].level, 0); // B
    }

    #[test]
    fn reorder_pure_ltr() {
        let resolver = BidiResolver::new(ParagraphDirection::Ltr);
        let levels = resolver.resolve_levels("ABC");
        let reordered = resolver.reorder(&levels);
        assert_eq!(reordered, vec!['A', 'B', 'C']);
    }

    #[test]
    fn reorder_empty() {
        let resolver = BidiResolver::new(ParagraphDirection::Ltr);
        let reordered = resolver.reorder(&[]);
        assert!(reordered.is_empty());
    }

    #[test]
    fn paragraph_direction_eq() {
        assert_eq!(ParagraphDirection::Ltr, ParagraphDirection::Ltr);
        assert_ne!(ParagraphDirection::Ltr, ParagraphDirection::Rtl);
    }
}

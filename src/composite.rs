//! 合成グリフ (Accented Letters)
//!
//! ベースグリフ + ダイアクリティカルマーク (アクセント記号) の合成。

use alloc::vec::Vec;

/// ダイアクリティカルマーク種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiacriticType {
    /// アキュートアクセント (´)。
    Acute,
    /// グレイヴアクセント (`` ` ``)。
    Grave,
    /// サーカムフレックス (^)。
    Circumflex,
    /// ウムラウト / ディエレシス (¨)。
    Diaeresis,
    /// チルダ (~)。
    Tilde,
    /// セディーユ (¸)。
    Cedilla,
    /// リング (°)。
    Ring,
    /// キャロン (ˇ)。
    Caron,
    /// マクロン (¯)。
    Macron,
}

/// ダイアクリティカルマークの配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiacriticPosition {
    /// 文字の上。
    Above,
    /// 文字の下。
    Below,
}

impl DiacriticType {
    /// Unicode combining character のコードポイント。
    #[must_use]
    pub const fn combining_codepoint(&self) -> u32 {
        match self {
            Self::Acute => 0x0301,
            Self::Grave => 0x0300,
            Self::Circumflex => 0x0302,
            Self::Diaeresis => 0x0308,
            Self::Tilde => 0x0303,
            Self::Cedilla => 0x0327,
            Self::Ring => 0x030A,
            Self::Caron => 0x030C,
            Self::Macron => 0x0304,
        }
    }

    /// デフォルト配置。
    #[must_use]
    pub const fn default_position(&self) -> DiacriticPosition {
        match self {
            Self::Cedilla => DiacriticPosition::Below,
            _ => DiacriticPosition::Above,
        }
    }
}

/// 合成グリフ定義。
#[derive(Debug, Clone)]
pub struct CompositeGlyph {
    /// ベース文字のコードポイント。
    pub base_char: char,
    /// ダイアクリティカルマークリスト。
    pub diacritics: Vec<DiacriticType>,
    /// 合成結果のコードポイント (既知の場合)。
    pub precomposed: Option<char>,
}

/// Unicode 分解テーブルエントリ。
struct DecompEntry {
    composed: char,
    base: char,
    diacritic: DiacriticType,
}

/// よく使われる合成文字の分解テーブル。
const DECOMP_TABLE: &[DecompEntry] = &[
    DecompEntry {
        composed: 'À',
        base: 'A',
        diacritic: DiacriticType::Grave,
    },
    DecompEntry {
        composed: 'Á',
        base: 'A',
        diacritic: DiacriticType::Acute,
    },
    DecompEntry {
        composed: 'Â',
        base: 'A',
        diacritic: DiacriticType::Circumflex,
    },
    DecompEntry {
        composed: 'Ã',
        base: 'A',
        diacritic: DiacriticType::Tilde,
    },
    DecompEntry {
        composed: 'Ä',
        base: 'A',
        diacritic: DiacriticType::Diaeresis,
    },
    DecompEntry {
        composed: 'Å',
        base: 'A',
        diacritic: DiacriticType::Ring,
    },
    DecompEntry {
        composed: 'Ç',
        base: 'C',
        diacritic: DiacriticType::Cedilla,
    },
    DecompEntry {
        composed: 'È',
        base: 'E',
        diacritic: DiacriticType::Grave,
    },
    DecompEntry {
        composed: 'É',
        base: 'E',
        diacritic: DiacriticType::Acute,
    },
    DecompEntry {
        composed: 'Ê',
        base: 'E',
        diacritic: DiacriticType::Circumflex,
    },
    DecompEntry {
        composed: 'Ë',
        base: 'E',
        diacritic: DiacriticType::Diaeresis,
    },
    DecompEntry {
        composed: 'Ñ',
        base: 'N',
        diacritic: DiacriticType::Tilde,
    },
    DecompEntry {
        composed: 'Ö',
        base: 'O',
        diacritic: DiacriticType::Diaeresis,
    },
    DecompEntry {
        composed: 'Ü',
        base: 'U',
        diacritic: DiacriticType::Diaeresis,
    },
    DecompEntry {
        composed: 'à',
        base: 'a',
        diacritic: DiacriticType::Grave,
    },
    DecompEntry {
        composed: 'á',
        base: 'a',
        diacritic: DiacriticType::Acute,
    },
    DecompEntry {
        composed: 'â',
        base: 'a',
        diacritic: DiacriticType::Circumflex,
    },
    DecompEntry {
        composed: 'ä',
        base: 'a',
        diacritic: DiacriticType::Diaeresis,
    },
    DecompEntry {
        composed: 'ç',
        base: 'c',
        diacritic: DiacriticType::Cedilla,
    },
    DecompEntry {
        composed: 'è',
        base: 'e',
        diacritic: DiacriticType::Grave,
    },
    DecompEntry {
        composed: 'é',
        base: 'e',
        diacritic: DiacriticType::Acute,
    },
    DecompEntry {
        composed: 'ê',
        base: 'e',
        diacritic: DiacriticType::Circumflex,
    },
    DecompEntry {
        composed: 'ë',
        base: 'e',
        diacritic: DiacriticType::Diaeresis,
    },
    DecompEntry {
        composed: 'ñ',
        base: 'n',
        diacritic: DiacriticType::Tilde,
    },
    DecompEntry {
        composed: 'ö',
        base: 'o',
        diacritic: DiacriticType::Diaeresis,
    },
    DecompEntry {
        composed: 'ü',
        base: 'u',
        diacritic: DiacriticType::Diaeresis,
    },
];

/// 合成文字を分解。
#[must_use]
pub fn decompose_char(c: char) -> Option<CompositeGlyph> {
    for entry in DECOMP_TABLE {
        if entry.composed == c {
            return Some(CompositeGlyph {
                base_char: entry.base,
                diacritics: alloc::vec![entry.diacritic],
                precomposed: Some(c),
            });
        }
    }
    None
}

/// 合成文字を検索 (ベース + ダイアクリティカル → 合成文字)。
#[must_use]
pub fn compose_char(base: char, diacritic: DiacriticType) -> Option<char> {
    for entry in DECOMP_TABLE {
        if entry.base == base && entry.diacritic == diacritic {
            return Some(entry.composed);
        }
    }
    None
}

/// 分解テーブルのエントリ数。
#[must_use]
pub const fn decomp_table_size() -> usize {
    DECOMP_TABLE.len()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_a_acute() {
        let result = decompose_char('á').unwrap();
        assert_eq!(result.base_char, 'a');
        assert_eq!(result.diacritics[0], DiacriticType::Acute);
    }

    #[test]
    fn decompose_u_diaeresis() {
        let result = decompose_char('ü').unwrap();
        assert_eq!(result.base_char, 'u');
        assert_eq!(result.diacritics[0], DiacriticType::Diaeresis);
    }

    #[test]
    fn decompose_c_cedilla() {
        let result = decompose_char('ç').unwrap();
        assert_eq!(result.base_char, 'c');
        assert_eq!(result.diacritics[0], DiacriticType::Cedilla);
    }

    #[test]
    fn decompose_ascii() {
        assert!(decompose_char('A').is_none());
    }

    #[test]
    fn compose_a_acute() {
        assert_eq!(compose_char('a', DiacriticType::Acute), Some('á'));
    }

    #[test]
    fn compose_not_found() {
        assert!(compose_char('z', DiacriticType::Ring).is_none());
    }

    #[test]
    fn diacritic_codepoint() {
        assert_eq!(DiacriticType::Acute.combining_codepoint(), 0x0301);
        assert_eq!(DiacriticType::Cedilla.combining_codepoint(), 0x0327);
    }

    #[test]
    fn diacritic_position() {
        assert_eq!(
            DiacriticType::Acute.default_position(),
            DiacriticPosition::Above
        );
        assert_eq!(
            DiacriticType::Cedilla.default_position(),
            DiacriticPosition::Below
        );
    }

    #[test]
    fn decomp_table_not_empty() {
        assert!(decomp_table_size() > 0);
    }

    #[test]
    fn roundtrip_decompose_compose() {
        let decomposed = decompose_char('É').unwrap();
        let recomposed = compose_char(decomposed.base_char, decomposed.diacritics[0]);
        assert_eq!(recomposed, Some('É'));
    }

    #[test]
    fn precomposed_set() {
        let result = decompose_char('ñ').unwrap();
        assert_eq!(result.precomposed, Some('ñ'));
    }

    #[test]
    fn diacritic_type_eq() {
        assert_eq!(DiacriticType::Acute, DiacriticType::Acute);
        assert_ne!(DiacriticType::Acute, DiacriticType::Grave);
    }

    #[test]
    fn all_uppercase_decomposable() {
        for &c in &[
            'À', 'Á', 'Â', 'Ã', 'Ä', 'Å', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ñ', 'Ö', 'Ü',
        ] {
            assert!(decompose_char(c).is_some(), "Failed for {c}");
        }
    }
}

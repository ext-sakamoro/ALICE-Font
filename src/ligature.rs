//! リガチャサポート
//!
//! 文字列置換ルールによるリガチャ (合字) 適用。

use alloc::vec::Vec;

/// リガチャルール。
#[derive(Debug, Clone)]
pub struct LigatureRule {
    /// 入力文字列。
    pub input: &'static str,
    /// 置換グリフ ID。
    pub replacement: u32,
    /// ルール名。
    pub name: &'static str,
}

/// 標準ラテンリガチャ。
pub const LATIN_LIGATURES: &[LigatureRule] = &[
    LigatureRule {
        input: "ffi",
        replacement: 0xFB03,
        name: "ffi",
    },
    LigatureRule {
        input: "ffl",
        replacement: 0xFB04,
        name: "ffl",
    },
    LigatureRule {
        input: "fi",
        replacement: 0xFB01,
        name: "fi",
    },
    LigatureRule {
        input: "fl",
        replacement: 0xFB02,
        name: "fl",
    },
    LigatureRule {
        input: "ff",
        replacement: 0xFB00,
        name: "ff",
    },
];

/// リガチャテーブル。
#[derive(Debug, Clone)]
pub struct LigatureTable {
    /// ルール (長い入力順にソート)。
    rules: Vec<LigatureRule>,
    /// 有効フラグ。
    enabled: bool,
}

impl LigatureTable {
    /// 標準ラテンリガチャテーブルを作成。
    #[must_use]
    pub fn latin() -> Self {
        let mut rules: Vec<LigatureRule> = LATIN_LIGATURES.to_vec();
        // 長い入力を先にマッチ
        rules.sort_by(|a, b| b.input.len().cmp(&a.input.len()));
        Self {
            rules,
            enabled: true,
        }
    }

    /// 空のテーブルを作成。
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            rules: Vec::new(),
            enabled: true,
        }
    }

    /// ルールを追加。
    pub fn add_rule(&mut self, rule: LigatureRule) {
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.input.len().cmp(&a.input.len()));
    }

    /// ルール数。
    #[must_use]
    pub const fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// 有効/無効を設定。
    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 有効か。
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// テキストにリガチャを適用。
    ///
    /// 置換されたグリフ ID のリストを返す。
    #[must_use]
    pub fn apply(&self, text: &str) -> Vec<LigatureResult> {
        if !self.enabled || self.rules.is_empty() {
            return text.chars().map(LigatureResult::Char).collect();
        }

        let bytes = text.as_bytes();
        let mut results = Vec::new();
        let mut i = 0;

        while i < bytes.len() {
            let mut matched = false;
            for rule in &self.rules {
                let input_bytes = rule.input.as_bytes();
                if i + input_bytes.len() <= bytes.len()
                    && &bytes[i..i + input_bytes.len()] == input_bytes
                {
                    results.push(LigatureResult::Ligature {
                        glyph_id: rule.replacement,
                        source_len: input_bytes.len(),
                    });
                    i += input_bytes.len();
                    matched = true;
                    break;
                }
            }
            if !matched {
                // UTF-8 文字境界を考慮
                let remaining = &text[i..];
                if let Some(c) = remaining.chars().next() {
                    results.push(LigatureResult::Char(c));
                    i += c.len_utf8();
                } else {
                    i += 1;
                }
            }
        }

        results
    }
}

/// リガチャ適用結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LigatureResult {
    /// 通常文字。
    Char(char),
    /// リガチャ置換。
    Ligature {
        /// グリフ ID。
        glyph_id: u32,
        /// 元の文字数。
        source_len: usize,
    },
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_table_has_rules() {
        let table = LigatureTable::latin();
        assert_eq!(table.rule_count(), 5);
    }

    #[test]
    fn apply_fi_ligature() {
        let table = LigatureTable::latin();
        let results = table.apply("find");
        assert!(matches!(
            results[0],
            LigatureResult::Ligature {
                glyph_id: 0xFB01,
                ..
            }
        ));
    }

    #[test]
    fn apply_ffi_ligature() {
        let table = LigatureTable::latin();
        let results = table.apply("office");
        // "o" + "ffi" ligature + "ce"
        let has_ffi = results.iter().any(|r| {
            matches!(
                r,
                LigatureResult::Ligature {
                    glyph_id: 0xFB03,
                    ..
                }
            )
        });
        assert!(has_ffi);
    }

    #[test]
    fn no_ligature() {
        let table = LigatureTable::latin();
        let results = table.apply("hello");
        assert!(results.iter().all(|r| matches!(r, LigatureResult::Char(_))));
    }

    #[test]
    fn disabled_table() {
        let mut table = LigatureTable::latin();
        table.set_enabled(false);
        let results = table.apply("find");
        assert!(results.iter().all(|r| matches!(r, LigatureResult::Char(_))));
    }

    #[test]
    fn empty_table() {
        let table = LigatureTable::empty();
        assert_eq!(table.rule_count(), 0);
    }

    #[test]
    fn empty_input() {
        let table = LigatureTable::latin();
        let results = table.apply("");
        assert!(results.is_empty());
    }

    #[test]
    fn add_custom_rule() {
        let mut table = LigatureTable::empty();
        table.add_rule(LigatureRule {
            input: "st",
            replacement: 0xFB06,
            name: "st",
        });
        assert_eq!(table.rule_count(), 1);
        let results = table.apply("stop");
        assert!(matches!(results[0], LigatureResult::Ligature { .. }));
    }

    #[test]
    fn longer_match_first() {
        let table = LigatureTable::latin();
        let results = table.apply("ffi");
        // "ffi" (3文字) が "ff" + "i" より優先
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0],
            LigatureResult::Ligature {
                glyph_id: 0xFB03,
                ..
            }
        ));
    }

    #[test]
    fn multiple_ligatures() {
        let table = LigatureTable::latin();
        let results = table.apply("fifi");
        let lig_count = results
            .iter()
            .filter(|r| matches!(r, LigatureResult::Ligature { .. }))
            .count();
        assert_eq!(lig_count, 2);
    }

    #[test]
    fn is_enabled_default() {
        let table = LigatureTable::latin();
        assert!(table.is_enabled());
    }

    #[test]
    fn ligature_result_eq() {
        assert_eq!(LigatureResult::Char('a'), LigatureResult::Char('a'));
        assert_ne!(LigatureResult::Char('a'), LigatureResult::Char('b'));
    }

    #[test]
    fn ff_ligature() {
        let table = LigatureTable::latin();
        let results = table.apply("off");
        let has_ff = results.iter().any(|r| {
            matches!(
                r,
                LigatureResult::Ligature {
                    glyph_id: 0xFB00,
                    ..
                }
            )
        });
        assert!(has_ff);
    }
}

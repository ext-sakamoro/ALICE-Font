//! CJK 拡張 — Radical-Stroke 合成フレームワーク
//!
//! 214 康熙部首 + IDS (Ideographic Description Sequence) 合成エンジン。
//!
//! 構成:
//! - [`radicals`] — 214 康熙部首テーブル
//! - [`layout`] — IDS 12 合成オペレータ
//! - [`ids`] — IDS 文字列パーサ
//! - [`ids_db`] — 常用漢字 IDS テーブル (S6 で拡充)
//!
//! License: MIT
//! Author: Moroya Sakamoto

pub mod ids;
pub mod ids_db;
pub mod layout;
pub mod radicals;

pub use layout::CompositionLayout;
pub use radicals::{
    find_radical_by_name, lookup_radical, radical_count, radicals_by_stroke_count, CjkRadical,
    RadicalId, RADICALS,
};

/// 漢字合成定義 (旧 API、互換のため残置)。
///
/// 新しいコードでは [`ids::Ids`] と [`ids_db`] を使うこと。
#[derive(Debug, Clone)]
pub struct CharComposition {
    /// 結果文字 (Unicode コードポイント)。
    pub result: char,
    /// 部首 ID。
    pub radical_id: RadicalId,
    /// 残りの部品 (文字)。
    pub component: char,
    /// レイアウト。
    pub layout: CompositionLayout,
    /// 総画数。
    pub total_strokes: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_lookup_radical_by_id() {
        let r = lookup_radical(1).unwrap();
        assert_eq!(r.character, '一');
        assert_eq!(r.stroke_count, 1);
    }

    #[test]
    fn legacy_lookup_radical_not_found() {
        assert!(lookup_radical(999).is_none());
    }

    #[test]
    fn legacy_find_by_name() {
        let r = find_radical_by_name("water").unwrap();
        assert_eq!(r.id, 85);
        assert_eq!(r.character, '水');
    }

    #[test]
    fn legacy_find_by_name_not_found() {
        assert!(find_radical_by_name("nonexistent").is_none());
    }

    #[test]
    fn legacy_radicals_by_strokes() {
        let result = radicals_by_stroke_count(3);
        assert!(!result.is_empty());
        assert!(result.iter().all(|r| r.stroke_count == 3));
    }

    #[test]
    fn legacy_radicals_by_strokes_none() {
        let result = radicals_by_stroke_count(20);
        assert!(result.is_empty());
    }

    #[test]
    fn legacy_radical_count_nonzero() {
        assert!(radical_count() > 0);
    }

    #[test]
    fn legacy_radical_table_sorted_ids() {
        for window in RADICALS.windows(2) {
            assert!(window[0].id <= window[1].id);
        }
    }

    #[test]
    fn legacy_composition_layout_eq() {
        assert_eq!(CompositionLayout::LeftRight, CompositionLayout::LeftRight);
        assert_ne!(CompositionLayout::LeftRight, CompositionLayout::TopBottom);
    }

    #[test]
    fn legacy_char_composition_struct() {
        let comp = CharComposition {
            result: '明',
            radical_id: 72,
            component: '月',
            layout: CompositionLayout::LeftRight,
            total_strokes: 8,
        };
        assert_eq!(comp.result, '明');
        assert_eq!(comp.total_strokes, 8);
    }
}

//! 常用漢字 IDS テーブル — S6 で本格拡充される。
//!
//! S5 完了時点では、合成エンジンと IDS パーサの動作確認用に最小限の
//! 漢字を手書き登録している。S6 で cjkvi-ids データから 2,136 字を
//! 自動生成し、ここに埋め込む計画。
//!
//! License: MIT
//! Author: Moroya Sakamoto

/// 単一漢字の IDS 定義。
#[derive(Debug, Clone, Copy)]
pub struct KanjiDef {
    /// 結果文字。
    pub codepoint: char,
    /// IDS 文字列 (合成構造)。
    pub ids: &'static str,
    /// 総画数。
    pub stroke_count: u8,
    /// 教育漢字学年 (1-6)。中学・高校は `None`。
    pub joyo_grade: Option<u8>,
}

/// S5 用の最小限の漢字テーブル — エンジン動作確認用。
/// S6 で `cjkvi-ids` ベースの 2,136 字テーブルに置き換える。
pub const KANJI_DB: &[KanjiDef] = &[
    KanjiDef {
        codepoint: '明',
        ids: "⿰日月",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '林',
        ids: "⿰木木",
        stroke_count: 8,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '森',
        ids: "⿱木⿰木木",
        stroke_count: 12,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '好',
        ids: "⿰女子",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '品',
        ids: "⿱口⿰口口",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '男',
        ids: "⿱田力",
        stroke_count: 7,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '国',
        ids: "⿴囗玉",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '同',
        ids: "⿵冂一",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
];

/// `ch` を結果文字に持つ漢字定義を返す。
#[must_use]
pub fn lookup(ch: char) -> Option<&'static KanjiDef> {
    KANJI_DB.iter().find(|d| d.codepoint == ch)
}

/// このバージョンで登録済みの漢字数。
#[must_use]
pub const fn registered_count() -> usize {
    KANJI_DB.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cjk::ids::parse;

    #[test]
    fn all_seed_kanji_have_valid_ids() {
        for def in KANJI_DB {
            let result = parse(def.ids);
            assert!(
                result.is_ok(),
                "IDS parse failed for {}: {:?} (ids = {})",
                def.codepoint,
                result.err(),
                def.ids
            );
        }
    }

    #[test]
    fn lookup_finds_mei() {
        let d = lookup('明').unwrap();
        assert_eq!(d.codepoint, '明');
        assert_eq!(d.ids, "⿰日月");
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        assert!(lookup('Z').is_none());
    }

    #[test]
    fn registered_count_is_nonzero() {
        assert!(registered_count() > 0);
    }
}

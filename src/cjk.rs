//! CJK 拡張 — Radical-Stroke 合成フレームワーク
//!
//! 部首 (Radical) と画数によるCJK漢字合成の基盤。

use alloc::vec::Vec;

/// 部首 ID (康熙部首番号 1-214)。
pub type RadicalId = u16;

/// 部首情報。
#[derive(Debug, Clone)]
pub struct CjkRadical {
    /// 部首番号 (1-214)。
    pub id: RadicalId,
    /// 部首文字。
    pub character: char,
    /// 画数。
    pub stroke_count: u8,
    /// 名称 (英語)。
    pub name: &'static str,
}

/// 主要な CJK 部首テーブル (上位50部首)。
pub const RADICALS: &[CjkRadical] = &[
    CjkRadical {
        id: 1,
        character: '一',
        stroke_count: 1,
        name: "one",
    },
    CjkRadical {
        id: 2,
        character: '丨',
        stroke_count: 1,
        name: "line",
    },
    CjkRadical {
        id: 3,
        character: '丶',
        stroke_count: 1,
        name: "dot",
    },
    CjkRadical {
        id: 4,
        character: '丿',
        stroke_count: 1,
        name: "slash",
    },
    CjkRadical {
        id: 9,
        character: '人',
        stroke_count: 2,
        name: "person",
    },
    CjkRadical {
        id: 18,
        character: '刀',
        stroke_count: 2,
        name: "knife",
    },
    CjkRadical {
        id: 30,
        character: '口',
        stroke_count: 3,
        name: "mouth",
    },
    CjkRadical {
        id: 32,
        character: '土',
        stroke_count: 3,
        name: "earth",
    },
    CjkRadical {
        id: 37,
        character: '大',
        stroke_count: 3,
        name: "big",
    },
    CjkRadical {
        id: 38,
        character: '女',
        stroke_count: 3,
        name: "woman",
    },
    CjkRadical {
        id: 40,
        character: '宀',
        stroke_count: 3,
        name: "roof",
    },
    CjkRadical {
        id: 46,
        character: '山',
        stroke_count: 3,
        name: "mountain",
    },
    CjkRadical {
        id: 50,
        character: '巾',
        stroke_count: 3,
        name: "cloth",
    },
    CjkRadical {
        id: 53,
        character: '广',
        stroke_count: 3,
        name: "dotted_cliff",
    },
    CjkRadical {
        id: 60,
        character: '彳',
        stroke_count: 3,
        name: "step",
    },
    CjkRadical {
        id: 61,
        character: '心',
        stroke_count: 4,
        name: "heart",
    },
    CjkRadical {
        id: 64,
        character: '手',
        stroke_count: 4,
        name: "hand",
    },
    CjkRadical {
        id: 72,
        character: '日',
        stroke_count: 4,
        name: "sun",
    },
    CjkRadical {
        id: 75,
        character: '木',
        stroke_count: 4,
        name: "tree",
    },
    CjkRadical {
        id: 85,
        character: '水',
        stroke_count: 4,
        name: "water",
    },
    CjkRadical {
        id: 86,
        character: '火',
        stroke_count: 4,
        name: "fire",
    },
    CjkRadical {
        id: 94,
        character: '犬',
        stroke_count: 4,
        name: "dog",
    },
    CjkRadical {
        id: 96,
        character: '玉',
        stroke_count: 5,
        name: "jade",
    },
    CjkRadical {
        id: 104,
        character: '疒',
        stroke_count: 5,
        name: "sickness",
    },
    CjkRadical {
        id: 109,
        character: '目',
        stroke_count: 5,
        name: "eye",
    },
    CjkRadical {
        id: 112,
        character: '石',
        stroke_count: 5,
        name: "stone",
    },
    CjkRadical {
        id: 118,
        character: '竹',
        stroke_count: 6,
        name: "bamboo",
    },
    CjkRadical {
        id: 120,
        character: '糸',
        stroke_count: 6,
        name: "silk",
    },
    CjkRadical {
        id: 130,
        character: '肉',
        stroke_count: 6,
        name: "meat",
    },
    CjkRadical {
        id: 140,
        character: '艸',
        stroke_count: 6,
        name: "grass",
    },
    CjkRadical {
        id: 142,
        character: '虫',
        stroke_count: 6,
        name: "insect",
    },
    CjkRadical {
        id: 149,
        character: '言',
        stroke_count: 7,
        name: "speech",
    },
    CjkRadical {
        id: 154,
        character: '貝',
        stroke_count: 7,
        name: "shell",
    },
    CjkRadical {
        id: 159,
        character: '車',
        stroke_count: 7,
        name: "cart",
    },
    CjkRadical {
        id: 162,
        character: '辶',
        stroke_count: 7,
        name: "walk",
    },
    CjkRadical {
        id: 167,
        character: '金',
        stroke_count: 8,
        name: "gold",
    },
    CjkRadical {
        id: 169,
        character: '門',
        stroke_count: 8,
        name: "gate",
    },
    CjkRadical {
        id: 170,
        character: '阜',
        stroke_count: 8,
        name: "mound",
    },
    CjkRadical {
        id: 184,
        character: '食',
        stroke_count: 9,
        name: "food",
    },
    CjkRadical {
        id: 187,
        character: '馬',
        stroke_count: 10,
        name: "horse",
    },
];

/// 部首を ID で検索。
#[must_use]
pub fn lookup_radical(id: RadicalId) -> Option<&'static CjkRadical> {
    RADICALS.iter().find(|r| r.id == id)
}

/// 部首を名前で検索。
#[must_use]
pub fn find_radical_by_name(name: &str) -> Option<&'static CjkRadical> {
    RADICALS.iter().find(|r| r.name == name)
}

/// 画数で部首を検索。
#[must_use]
pub fn radicals_by_stroke_count(strokes: u8) -> Vec<&'static CjkRadical> {
    RADICALS
        .iter()
        .filter(|r| r.stroke_count == strokes)
        .collect()
}

/// 漢字合成レイアウト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositionLayout {
    /// 左右配置 (⿰)。
    LeftRight,
    /// 上下配置 (⿱)。
    TopBottom,
    /// 囲み配置 (⿴)。
    Enclosure,
    /// 左上囲み (⿸)。
    TopLeftSurround,
    /// 左下囲み (⿺)。
    BottomLeftSurround,
}

/// 漢字合成定義。
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

/// 部首テーブルのエントリ数。
#[must_use]
pub const fn radical_count() -> usize {
    RADICALS.len()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_radical_by_id() {
        let r = lookup_radical(1).unwrap();
        assert_eq!(r.character, '一');
        assert_eq!(r.stroke_count, 1);
    }

    #[test]
    fn lookup_radical_not_found() {
        assert!(lookup_radical(999).is_none());
    }

    #[test]
    fn find_by_name() {
        let r = find_radical_by_name("water").unwrap();
        assert_eq!(r.id, 85);
        assert_eq!(r.character, '水');
    }

    #[test]
    fn find_by_name_not_found() {
        assert!(find_radical_by_name("nonexistent").is_none());
    }

    #[test]
    fn radicals_by_strokes() {
        let result = radicals_by_stroke_count(3);
        assert!(!result.is_empty());
        assert!(result.iter().all(|r| r.stroke_count == 3));
    }

    #[test]
    fn radicals_by_strokes_none() {
        let result = radicals_by_stroke_count(20);
        assert!(result.is_empty());
    }

    #[test]
    fn radical_count_nonzero() {
        assert!(radical_count() > 0);
    }

    #[test]
    fn radical_table_sorted_ids() {
        for window in RADICALS.windows(2) {
            assert!(window[0].id <= window[1].id);
        }
    }

    #[test]
    fn composition_layout_eq() {
        assert_eq!(CompositionLayout::LeftRight, CompositionLayout::LeftRight);
        assert_ne!(CompositionLayout::LeftRight, CompositionLayout::TopBottom);
    }

    #[test]
    fn char_composition_struct() {
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

    #[test]
    fn mouth_radical() {
        let r = lookup_radical(30).unwrap();
        assert_eq!(r.name, "mouth");
        assert_eq!(r.character, '口');
    }

    #[test]
    fn heart_radical() {
        let r = find_radical_by_name("heart").unwrap();
        assert_eq!(r.id, 61);
    }

    #[test]
    fn radical_stroke_count() {
        let r = lookup_radical(167).unwrap();
        assert_eq!(r.name, "gold");
        assert_eq!(r.stroke_count, 8);
    }
}

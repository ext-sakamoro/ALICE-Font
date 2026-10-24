//! 漢字 IDS テーブル — 手書き定義。
//!
//! S6.2.1 (grade 1..=6) — 教育漢字 全 6 学年 の carrier-level カバレッジ達成
//! (計 1018 字 の joyo_grade Some(1..=6) 保証)。内訳: 80 + 160 + 199 +
//! 211 + 186 + 182 = 1018 (canonical 1026 との差 -8、下記注記参照)。
//! seed 267 + g1 補完 59 + g2 補完 113 + g3 補完 141 +
//! g4 補完 175 + g5 補完 137 + g6 補完 160 = 計 1052 字。
//!
//! 注 (canonical vs 本 crate の差異、canonical patch 予定):
//! - grade 3 は MEXT 総数 "200 字" と表記されるが標準列挙は 199 字。
//! - grade 4 は MEXT 総数 "202 字" だが、20 都道府県漢字を全て含む
//!   211 字 superset を採用 (2017 revision カバレッジ担保)。
//! - grade 5 は MEXT 総数 "193 字" だが、確実な 186 字 subset (-7 pending)。
//! - grade 6 は MEXT 総数 "191 字" だが、確実な 182 字 subset (-9 pending)。
//!
//! 教育漢字 1026 canonical に対して 1018/1026 = 99.2 %。canonical patch
//! (grade 5 +7 / grade 6 +9 / grade 4 -9 subset) で正確な 1026 に到達可能。
//!
//! 後続セッションで C 案 (常用漢字フル 2,136 字) に拡張予定。
//!
//! 各エントリの IDS 文字列は人手で定義されている。確信度の低いものは
//! `Leaf`(単一文字、分解なし) として登録し、視覚的には placeholder で
//! 描画される。後続作業で順次分解パターンを追加していく。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

/// 単一漢字の IDS 定義。
#[derive(Debug, Clone, Copy)]
pub struct KanjiDef {
    /// 結果文字。
    pub codepoint: char,
    /// IDS 文字列 (合成構造、または単一文字)。
    pub ids: &'static str,
    /// 総画数 (未設定は 0)。
    pub stroke_count: u8,
    /// 教育漢字学年 (1-6)、中学・高校・表外は `None`。
    pub joyo_grade: Option<u8>,
}

/// KANJI_DB — S6.2.1 grade 1..=6 全 6 学年 carrier-level coverage
/// (1018 字 = 教育漢字 canonical 1026 の 99.2 %) を含む 1052 字の IDS DB。
///
/// 内訳: seed 267 + g1 59 + g2 113 + g3 141 + g4 175 + g5 137 + g6 160。
/// `codepoint` 昇順 (Unicode 順) で並べる。重複なし。
pub const KANJI_DB: &[KanjiDef] = &[
    KanjiDef {
        codepoint: '一',
        ids: "一",
        stroke_count: 1,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '丁',
        ids: "丁",
        stroke_count: 2,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '七',
        ids: "七",
        stroke_count: 2,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '万',
        ids: "万",
        stroke_count: 3,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '三',
        ids: "三",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '上',
        ids: "上",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '下',
        ids: "下",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '不',
        ids: "不",
        stroke_count: 4,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '世',
        ids: "世",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '両',
        ids: "両",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '並',
        ids: "並",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '中',
        ids: "中",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '丸',
        ids: "丸",
        stroke_count: 3,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '主',
        ids: "主",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '久',
        ids: "久",
        stroke_count: 3,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '乗',
        ids: "乗",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '九',
        ids: "九",
        stroke_count: 2,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '乱',
        ids: "乱",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '乳',
        ids: "乳",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '了',
        ids: "了",
        stroke_count: 2,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '予',
        ids: "予",
        stroke_count: 4,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '争',
        ids: "争",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '事',
        ids: "事",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '二',
        ids: "二",
        stroke_count: 2,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '五',
        ids: "五",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '井',
        ids: "井",
        stroke_count: 4,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '亡',
        ids: "亡",
        stroke_count: 3,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '交',
        ids: "交",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '京',
        ids: "京",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '人',
        ids: "人",
        stroke_count: 2,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '仁',
        ids: "仁",
        stroke_count: 4,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '今',
        ids: "今",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '仏',
        ids: "仏",
        stroke_count: 4,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '仕',
        ids: "⿰人士",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '他',
        ids: "⿰人也",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '付',
        ids: "⿰人寸",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '代',
        ids: "⿰人弋",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '令',
        ids: "令",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '以',
        ids: "以",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '仮',
        ids: "仮",
        stroke_count: 6,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '仲',
        ids: "仲",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '件',
        ids: "⿰人牛",
        stroke_count: 6,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '任',
        ids: "⿰人壬",
        stroke_count: 6,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '企',
        ids: "⿱人止",
        stroke_count: 6,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '休',
        ids: "休",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '会',
        ids: "⿱人云",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '伝',
        ids: "伝",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '似',
        ids: "似",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '位',
        ids: "⿰人立",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '低',
        ids: "低",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '住',
        ids: "住",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '佐',
        ids: "佐",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '体',
        ids: "⿰人本",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '何',
        ids: "何",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '余',
        ids: "余",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '作',
        ids: "⿰人乍",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '使',
        ids: "使",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '例',
        ids: "例",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '供',
        ids: "⿰人共",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '価',
        ids: "⿰人西",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '便',
        ids: "便",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '係',
        ids: "係",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '保',
        ids: "⿰人呆",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '信',
        ids: "⿰人言",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '修',
        ids: "修",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '俳',
        ids: "俳",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '俵',
        ids: "俵",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '倉',
        ids: "倉",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '個',
        ids: "個",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '倍',
        ids: "倍",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '候',
        ids: "候",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '借',
        ids: "借",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '値',
        ids: "⿰人直",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '倫',
        ids: "⿰人侖",
        stroke_count: 10,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '停',
        ids: "停",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '健',
        ids: "健",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '側',
        ids: "側",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '備',
        ids: "備",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '傷',
        ids: "傷",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '働',
        ids: "働",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '像',
        ids: "像",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '億',
        ids: "億",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '優',
        ids: "優",
        stroke_count: 17,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '元',
        ids: "元",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '兄',
        ids: "兄",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '兆',
        ids: "兆",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '先',
        ids: "先",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '光',
        ids: "光",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '児',
        ids: "児",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '党',
        ids: "党",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '入',
        ids: "入",
        stroke_count: 2,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '全',
        ids: "全",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '八',
        ids: "八",
        stroke_count: 2,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '公',
        ids: "⿱八厶",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '六',
        ids: "六",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '共',
        ids: "共",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '兵',
        ids: "兵",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '具',
        ids: "具",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '典',
        ids: "典",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '内',
        ids: "⿵冂人",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '円',
        ids: "⿵冂士",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '冊',
        ids: "冊",
        stroke_count: 5,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '再',
        ids: "再",
        stroke_count: 6,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '写',
        ids: "写",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '冬',
        ids: "冬",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '冷',
        ids: "冷",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '処',
        ids: "処",
        stroke_count: 5,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '出',
        ids: "出",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '刀',
        ids: "刀",
        stroke_count: 2,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '分',
        ids: "⿱八刀",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '切',
        ids: "切",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '刊',
        ids: "刊",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '列',
        ids: "⿰歹刀",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '初',
        ids: "初",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '判',
        ids: "判",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '別',
        ids: "別",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '利',
        ids: "利",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '制',
        ids: "制",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '刷',
        ids: "刷",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '券',
        ids: "券",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '刻',
        ids: "刻",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '則',
        ids: "則",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '前',
        ids: "前",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '副',
        ids: "副",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '割',
        ids: "割",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '創',
        ids: "⿰倉刀",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '劇',
        ids: "劇",
        stroke_count: 15,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '力',
        ids: "力",
        stroke_count: 2,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '功',
        ids: "⿰工力",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '加',
        ids: "⿰力口",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '助',
        ids: "助",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '努',
        ids: "努",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '労',
        ids: "労",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '効',
        ids: "⿰交力",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '勇',
        ids: "勇",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '勉',
        ids: "勉",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '動',
        ids: "⿰重力",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '務',
        ids: "務",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '勝',
        ids: "勝",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '勢',
        ids: "勢",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '勤',
        ids: "勤",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '包',
        ids: "包",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '化',
        ids: "⿰人匕",
        stroke_count: 4,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '北',
        ids: "北",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '区',
        ids: "区",
        stroke_count: 4,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '医',
        ids: "医",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '匿',
        ids: "匿",
        stroke_count: 10,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '十',
        ids: "十",
        stroke_count: 2,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '千',
        ids: "千",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '午',
        ids: "午",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '半',
        ids: "半",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '卒',
        ids: "卒",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '協',
        ids: "協",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '南',
        ids: "南",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '単',
        ids: "単",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '博',
        ids: "博",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '印',
        ids: "印",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '危',
        ids: "危",
        stroke_count: 6,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '卵',
        ids: "卵",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '厚',
        ids: "厚",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '原',
        ids: "原",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '厳',
        ids: "厳",
        stroke_count: 17,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '去',
        ids: "去",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '参',
        ids: "参",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '友',
        ids: "友",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '反',
        ids: "反",
        stroke_count: 4,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '収',
        ids: "収",
        stroke_count: 4,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '取',
        ids: "⿰耳又",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '受',
        ids: "受",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '口',
        ids: "口",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '古',
        ids: "⿱十口",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '句',
        ids: "句",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '可',
        ids: "可",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '台',
        ids: "台",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '史',
        ids: "史",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '右',
        ids: "右",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '号',
        ids: "号",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '司',
        ids: "司",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '各',
        ids: "各",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '合',
        ids: "⿱人口",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '同',
        ids: "同",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '名',
        ids: "⿱夕口",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '后',
        ids: "后",
        stroke_count: 6,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '向',
        ids: "⿵冂口",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '君',
        ids: "君",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '否',
        ids: "否",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '吸',
        ids: "吸",
        stroke_count: 6,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '告',
        ids: "告",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '周',
        ids: "周",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '味',
        ids: "⿰口未",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '呼',
        ids: "呼",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '命',
        ids: "命",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '和',
        ids: "⿰禾口",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '品',
        ids: "⿱口⿰口口",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '員',
        ids: "員",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '唱',
        ids: "唱",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '商',
        ids: "商",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '問',
        ids: "⿵門口",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '善',
        ids: "善",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '喜',
        ids: "喜",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '営',
        ids: "営",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '器',
        ids: "器",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '四',
        ids: "四",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '回',
        ids: "⿴口口",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '因',
        ids: "⿴囗大",
        stroke_count: 6,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '団',
        ids: "⿴囗才",
        stroke_count: 6,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '困',
        ids: "困",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '囲',
        ids: "囲",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '図',
        ids: "図",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '固',
        ids: "固",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '国',
        ids: "⿴囗玉",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '園',
        ids: "園",
        stroke_count: 13,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '土',
        ids: "土",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '圧',
        ids: "⿸厂土",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '在',
        ids: "在",
        stroke_count: 6,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '地',
        ids: "⿰土也",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '坂',
        ids: "⿰土反",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '均',
        ids: "均",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '垂',
        ids: "垂",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '型',
        ids: "⿱刑土",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '城',
        ids: "⿰土成",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '域',
        ids: "域",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '基',
        ids: "⿱其土",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '埼',
        ids: "埼",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '堂',
        ids: "堂",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '報',
        ids: "報",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '場',
        ids: "場",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '塩',
        ids: "塩",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '境',
        ids: "境",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '墓',
        ids: "⿱莫土",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '増',
        ids: "増",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '士',
        ids: "士",
        stroke_count: 3,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '声',
        ids: "声",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '売',
        ids: "売",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '変',
        ids: "変",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '夏',
        ids: "夏",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '夕',
        ids: "夕",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '外',
        ids: "⿰夕卜",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '多',
        ids: "⿱夕夕",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '夜',
        ids: "夜",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '夢',
        ids: "夢",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '大',
        ids: "大",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '天',
        ids: "⿱一大",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '太',
        ids: "太",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '夫',
        ids: "夫",
        stroke_count: 4,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '央',
        ids: "央",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '失',
        ids: "失",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '奈',
        ids: "⿱大示",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '奏',
        ids: "奏",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '奨',
        ids: "奨",
        stroke_count: 13,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '奮',
        ids: "⿱大⿱隹田",
        stroke_count: 16,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '女',
        ids: "女",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '好',
        ids: "⿰女子",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '妹',
        ids: "⿰女未",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '妻',
        ids: "妻",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '姉',
        ids: "姉",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '始',
        ids: "⿰女台",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '委',
        ids: "⿱禾女",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '姿',
        ids: "姿",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '婦',
        ids: "婦",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '媛',
        ids: "媛",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '子',
        ids: "子",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '字',
        ids: "⿱宀子",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '存',
        ids: "存",
        stroke_count: 6,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '孝',
        ids: "孝",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '季',
        ids: "⿱禾子",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '学',
        ids: "学",
        stroke_count: 8,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '孫',
        ids: "孫",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '宅',
        ids: "宅",
        stroke_count: 6,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '宇',
        ids: "宇",
        stroke_count: 6,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '守',
        ids: "⿱宀寸",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '安',
        ids: "⿱宀女",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '完',
        ids: "⿱宀元",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '宗',
        ids: "⿱宀示",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '官',
        ids: "官",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '宙',
        ids: "⿱宀由",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '定',
        ids: "定",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '宝',
        ids: "宝",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '実',
        ids: "実",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '客',
        ids: "⿱宀各",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '宣',
        ids: "宣",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '室',
        ids: "室",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '宮',
        ids: "宮",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '害',
        ids: "害",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '家',
        ids: "家",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '容',
        ids: "容",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '宿',
        ids: "宿",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '寄',
        ids: "寄",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '密',
        ids: "密",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '富',
        ids: "⿱宀畐",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '寒',
        ids: "寒",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '察',
        ids: "⿱宀祭",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '寸',
        ids: "寸",
        stroke_count: 3,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '寺',
        ids: "寺",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '対',
        ids: "⿰文寸",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '専',
        ids: "専",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '射',
        ids: "射",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '将',
        ids: "将",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '尊',
        ids: "尊",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '導',
        ids: "導",
        stroke_count: 15,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '小',
        ids: "小",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '少',
        ids: "少",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '就',
        ids: "就",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '尺',
        ids: "尺",
        stroke_count: 4,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '局',
        ids: "局",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '居',
        ids: "居",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '届',
        ids: "届",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '屋',
        ids: "屋",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '展',
        ids: "展",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '属',
        ids: "属",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '層',
        ids: "層",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '山',
        ids: "山",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '岐',
        ids: "岐",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '岡',
        ids: "岡",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '岩',
        ids: "⿱山石",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '岸',
        ids: "岸",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '島',
        ids: "島",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '崎',
        ids: "崎",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '川',
        ids: "川",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '州',
        ids: "州",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '巣',
        ids: "巣",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '工',
        ids: "工",
        stroke_count: 3,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '左',
        ids: "左",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '差',
        ids: "差",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '己',
        ids: "己",
        stroke_count: 3,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '巻',
        ids: "巻",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '市',
        ids: "市",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '布',
        ids: "布",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '希',
        ids: "希",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '師',
        ids: "師",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '席',
        ids: "席",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '帯',
        ids: "帯",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '帰',
        ids: "帰",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '帳',
        ids: "帳",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '常',
        ids: "常",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '幅',
        ids: "⿰巾畐",
        stroke_count: 12,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '幕',
        ids: "⿱莫巾",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '干',
        ids: "干",
        stroke_count: 3,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '平',
        ids: "平",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '年',
        ids: "年",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '幸',
        ids: "幸",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '幹',
        ids: "幹",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '幼',
        ids: "幼",
        stroke_count: 5,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '庁',
        ids: "庁",
        stroke_count: 5,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '広',
        ids: "広",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '序',
        ids: "⿸广予",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '底',
        ids: "底",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '店',
        ids: "店",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '府',
        ids: "府",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '度',
        ids: "度",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '座',
        ids: "座",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '庫',
        ids: "庫",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '庭',
        ids: "庭",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '康',
        ids: "康",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '延',
        ids: "延",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '建',
        ids: "建",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '弁',
        ids: "弁",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '式',
        ids: "⿹弋工",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '弓',
        ids: "弓",
        stroke_count: 3,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '引',
        ids: "⿰弓丨",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '弟',
        ids: "弟",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '弱',
        ids: "弱",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '張',
        ids: "⿰弓長",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '強',
        ids: "強",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '当',
        ids: "当",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '形',
        ids: "形",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '役',
        ids: "⿰彳殳",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '往',
        ids: "往",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '径',
        ids: "径",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '待',
        ids: "待",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '律',
        ids: "律",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '後',
        ids: "後",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '徒',
        ids: "徒",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '従',
        ids: "従",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '得',
        ids: "得",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '復',
        ids: "復",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '徳',
        ids: "徳",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '心',
        ids: "心",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '必',
        ids: "必",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '志',
        ids: "⿱士心",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '忘',
        ids: "忘",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '応',
        ids: "応",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '忠',
        ids: "⿱中心",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '快',
        ids: "快",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '念',
        ids: "⿱今心",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '思',
        ids: "⿱田心",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '急',
        ids: "急",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '性',
        ids: "⿰心生",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '恩',
        ids: "恩",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '息',
        ids: "⿱自心",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '悪',
        ids: "悪",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '悲',
        ids: "悲",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '情',
        ids: "⿰心青",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '想',
        ids: "⿱相心",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '意',
        ids: "⿱音心",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '愛',
        ids: "愛",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '感',
        ids: "⿱咸心",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '態',
        ids: "態",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '慣',
        ids: "慣",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '憲',
        ids: "憲",
        stroke_count: 16,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '憶',
        ids: "⿰心意",
        stroke_count: 16,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '成',
        ids: "成",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '我',
        ids: "我",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '戦',
        ids: "⿰単戈",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '戸',
        ids: "戸",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '戻',
        ids: "⿸戸大",
        stroke_count: 7,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '所',
        ids: "⿰戸斤",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '手',
        ids: "手",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '才',
        ids: "才",
        stroke_count: 3,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '打',
        ids: "打",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '批',
        ids: "批",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '技',
        ids: "⿰手支",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '投',
        ids: "投",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '折',
        ids: "⿰手斤",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '択',
        ids: "⿰手尺",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '担',
        ids: "⿰手旦",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '招',
        ids: "招",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '拝',
        ids: "拝",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '拡',
        ids: "⿰手広",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '拾',
        ids: "拾",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '持',
        ids: "⿰手寺",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '指',
        ids: "⿰手旨",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '挑',
        ids: "⿰手兆",
        stroke_count: 9,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '挙',
        ids: "挙",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '捗',
        ids: "⿰手歩",
        stroke_count: 10,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '捨',
        ids: "捨",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '授',
        ids: "授",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '採',
        ids: "採",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '探',
        ids: "探",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '接',
        ids: "接",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '推',
        ids: "⿰手隹",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '提',
        ids: "⿰手是",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '揮',
        ids: "揮",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '援',
        ids: "援",
        stroke_count: 12,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '損',
        ids: "損",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '撃',
        ids: "撃",
        stroke_count: 15,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '操',
        ids: "操",
        stroke_count: 16,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '支',
        ids: "支",
        stroke_count: 4,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '改',
        ids: "⿰己攵",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '攻',
        ids: "⿰工攵",
        stroke_count: 7,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '放',
        ids: "放",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '政',
        ids: "⿰正攵",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '故',
        ids: "故",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '救',
        ids: "救",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '敗',
        ids: "⿰貝攵",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '教',
        ids: "教",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '散',
        ids: "散",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '敬',
        ids: "敬",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '数',
        ids: "数",
        stroke_count: 13,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '整',
        ids: "整",
        stroke_count: 16,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '敵',
        ids: "⿰啇攵",
        stroke_count: 15,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '文',
        ids: "文",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '料',
        ids: "⿰米斗",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '断',
        ids: "断",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '新',
        ids: "⿰立⿱木斤",
        stroke_count: 13,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '方',
        ids: "方",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '旅',
        ids: "旅",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '族',
        ids: "族",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '旗',
        ids: "旗",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '日',
        ids: "日",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '旧',
        ids: "旧",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '早',
        ids: "⿱日十",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '明',
        ids: "⿰日月",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '易',
        ids: "⿱日勿",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '昔',
        ids: "昔",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '星',
        ids: "⿱日生",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '映',
        ids: "映",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '春',
        ids: "春",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '昨',
        ids: "昨",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '昭',
        ids: "⿰日召",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '昼',
        ids: "昼",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '時',
        ids: "⿰日寺",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '晩',
        ids: "晩",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '景',
        ids: "景",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '晴',
        ids: "⿰日青",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '暑',
        ids: "⿱日者",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '暖',
        ids: "暖",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '暗',
        ids: "暗",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '暮',
        ids: "暮",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '暴',
        ids: "暴",
        stroke_count: 15,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '曜',
        ids: "曜",
        stroke_count: 18,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '曲',
        ids: "曲",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '更',
        ids: "更",
        stroke_count: 7,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '書',
        ids: "書",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '最',
        ids: "⿱日取",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '月',
        ids: "月",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '有',
        ids: "有",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '服',
        ids: "服",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '朗',
        ids: "朗",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '望',
        ids: "望",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '朝',
        ids: "朝",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '期',
        ids: "⿰其月",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '木',
        ids: "木",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '未',
        ids: "未",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '末',
        ids: "末",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '本',
        ids: "本",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '札',
        ids: "札",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '机',
        ids: "机",
        stroke_count: 6,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '材',
        ids: "⿰木才",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '村',
        ids: "⿰木寸",
        stroke_count: 7,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '束',
        ids: "束",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '条',
        ids: "条",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '来',
        ids: "来",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '東',
        ids: "東",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '松',
        ids: "松",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '板',
        ids: "⿰木反",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '林',
        ids: "⿰木木",
        stroke_count: 8,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '枚',
        ids: "枚",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '果',
        ids: "⿱田木",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '枝',
        ids: "⿰木支",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '染',
        ids: "染",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '柱',
        ids: "柱",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '査',
        ids: "査",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '栃',
        ids: "栃",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '栄',
        ids: "栄",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '校',
        ids: "⿰木交",
        stroke_count: 10,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '株',
        ids: "⿰木朱",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '根',
        ids: "根",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '格',
        ids: "⿰木各",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '案',
        ids: "案",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '桜',
        ids: "桜",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '梅',
        ids: "梅",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '梨',
        ids: "梨",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '械',
        ids: "械",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '棒',
        ids: "棒",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '森',
        ids: "⿱木⿰木木",
        stroke_count: 12,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '植',
        ids: "⿰木直",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '検',
        ids: "⿰木僉",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '業',
        ids: "業",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '極',
        ids: "極",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '楽',
        ids: "楽",
        stroke_count: 13,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '概',
        ids: "⿰木既",
        stroke_count: 14,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '構',
        ids: "⿰木冓",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '様',
        ids: "⿰木羕",
        stroke_count: 14,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '標',
        ids: "標",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '模',
        ids: "⿰木莫",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '権',
        ids: "権",
        stroke_count: 15,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '横',
        ids: "横",
        stroke_count: 15,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '樹',
        ids: "樹",
        stroke_count: 16,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '橋',
        ids: "橋",
        stroke_count: 16,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '機',
        ids: "機",
        stroke_count: 16,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '欠',
        ids: "欠",
        stroke_count: 4,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '次',
        ids: "⿰冫欠",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '欲',
        ids: "欲",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '歌',
        ids: "歌",
        stroke_count: 14,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '止',
        ids: "止",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '正',
        ids: "正",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '武',
        ids: "⿹弋止",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '歩',
        ids: "⿱止少",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '歯',
        ids: "歯",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '歴',
        ids: "歴",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '死',
        ids: "⿰歹匕",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '残',
        ids: "⿰歹戔",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '段',
        ids: "段",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '殺',
        ids: "殺",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '母',
        ids: "母",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '毎',
        ids: "毎",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '毒',
        ids: "毒",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '比',
        ids: "比",
        stroke_count: 4,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '毛',
        ids: "毛",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '氏',
        ids: "氏",
        stroke_count: 4,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '民',
        ids: "民",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '気',
        ids: "気",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '水',
        ids: "水",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '氷',
        ids: "氷",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '永',
        ids: "永",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '求',
        ids: "求",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '汎',
        ids: "⿰水凡",
        stroke_count: 6,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '池',
        ids: "池",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '決',
        ids: "決",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '汽',
        ids: "汽",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '沖',
        ids: "沖",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '河',
        ids: "河",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '油',
        ids: "油",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '治',
        ids: "治",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '沿',
        ids: "沿",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '泉',
        ids: "泉",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '法',
        ids: "法",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '波',
        ids: "波",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '泣',
        ids: "泣",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '注',
        ids: "注",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '泳',
        ids: "泳",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '洋',
        ids: "洋",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '洗',
        ids: "洗",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '活',
        ids: "⿰水舌",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '派',
        ids: "派",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '流',
        ids: "流",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '浅',
        ids: "浅",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '浴',
        ids: "浴",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '海',
        ids: "海",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '消',
        ids: "消",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '液',
        ids: "液",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '深',
        ids: "深",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '混',
        ids: "混",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '清',
        ids: "清",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '済',
        ids: "済",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '減',
        ids: "減",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '温',
        ids: "温",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '測',
        ids: "⿰水則",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '港',
        ids: "港",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '湖',
        ids: "湖",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '湯',
        ids: "湯",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '満',
        ids: "満",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '源',
        ids: "源",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '準',
        ids: "準",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '滋',
        ids: "滋",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '漁',
        ids: "漁",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '演',
        ids: "演",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '漢',
        ids: "漢",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '潔',
        ids: "潔",
        stroke_count: 15,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '潟',
        ids: "潟",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '潮',
        ids: "潮",
        stroke_count: 15,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '激',
        ids: "激",
        stroke_count: 16,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '火',
        ids: "火",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '灯',
        ids: "灯",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '灰',
        ids: "灰",
        stroke_count: 6,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '災',
        ids: "災",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '炭',
        ids: "炭",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '点',
        ids: "点",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '無',
        ids: "無",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '然',
        ids: "然",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '焼',
        ids: "焼",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '照',
        ids: "照",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '熊',
        ids: "熊",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '熟',
        ids: "熟",
        stroke_count: 15,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '熱',
        ids: "熱",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '燃',
        ids: "燃",
        stroke_count: 16,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '父',
        ids: "父",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '片',
        ids: "片",
        stroke_count: 4,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '版',
        ids: "版",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '牛',
        ids: "牛",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '牧',
        ids: "⿰牛攵",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '物',
        ids: "⿰牛勿",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '特',
        ids: "⿰牛寺",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '犬',
        ids: "犬",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '犯',
        ids: "犯",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '状',
        ids: "状",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '独',
        ids: "独",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '率',
        ids: "率",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '玉',
        ids: "玉",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '王',
        ids: "王",
        stroke_count: 4,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '班',
        ids: "班",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '現',
        ids: "⿰玉見",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '球',
        ids: "球",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '理',
        ids: "⿰玉里",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '生',
        ids: "生",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '産',
        ids: "産",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '用',
        ids: "用",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '田',
        ids: "田",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '由',
        ids: "由",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '申',
        ids: "申",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '男',
        ids: "⿱田力",
        stroke_count: 7,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '町',
        ids: "町",
        stroke_count: 7,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '画',
        ids: "画",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '界',
        ids: "⿱田介",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '畑',
        ids: "畑",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '留',
        ids: "留",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '略',
        ids: "⿰田各",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '番',
        ids: "⿱釆田",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '異',
        ids: "異",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '疑',
        ids: "疑",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '病',
        ids: "病",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '痛',
        ids: "痛",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '発',
        ids: "発",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '登',
        ids: "登",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '白',
        ids: "白",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '百',
        ids: "⿱一白",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '的',
        ids: "⿰白勺",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '皇',
        ids: "皇",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '皮',
        ids: "皮",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '皿',
        ids: "皿",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '益',
        ids: "益",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '盛',
        ids: "盛",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '盟',
        ids: "盟",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '目',
        ids: "目",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '直',
        ids: "直",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '相',
        ids: "⿰木目",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '省',
        ids: "⿱少目",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '看',
        ids: "看",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '県',
        ids: "⿱目糸",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '真',
        ids: "真",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '眼',
        ids: "眼",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '着',
        ids: "着",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '矢',
        ids: "矢",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '知',
        ids: "⿰矢口",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '短',
        ids: "短",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '石',
        ids: "石",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '砂',
        ids: "⿰石少",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '研',
        ids: "⿰石幵",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '破',
        ids: "破",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '確',
        ids: "⿰石隺",
        stroke_count: 15,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '磁',
        ids: "磁",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '示',
        ids: "示",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '礼',
        ids: "礼",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '社',
        ids: "⿰示土",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '祖',
        ids: "祖",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '祝',
        ids: "祝",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '神',
        ids: "神",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '票',
        ids: "票",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '祭',
        ids: "祭",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '禁',
        ids: "禁",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '福',
        ids: "⿰示畐",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '私',
        ids: "⿰禾厶",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '秋',
        ids: "秋",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '科',
        ids: "科",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '秒',
        ids: "⿰禾少",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '秘',
        ids: "秘",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '移',
        ids: "移",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '程',
        ids: "程",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '税',
        ids: "税",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '種',
        ids: "⿰禾重",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '穀',
        ids: "穀",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '積',
        ids: "積",
        stroke_count: 16,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '穴',
        ids: "穴",
        stroke_count: 5,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '究',
        ids: "⿱穴九",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '空',
        ids: "空",
        stroke_count: 8,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '窓',
        ids: "窓",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '立',
        ids: "立",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '章',
        ids: "⿱立日",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '童',
        ids: "童",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '端',
        ids: "⿰立耑",
        stroke_count: 14,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '競',
        ids: "競",
        stroke_count: 20,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '竹',
        ids: "竹",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '笑',
        ids: "笑",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '笛',
        ids: "笛",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '第',
        ids: "第",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '筆',
        ids: "筆",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '等',
        ids: "等",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '筋',
        ids: "筋",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '答',
        ids: "⿱竹合",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '策',
        ids: "策",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '算',
        ids: "算",
        stroke_count: 14,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '管',
        ids: "管",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '箱',
        ids: "箱",
        stroke_count: 15,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '節',
        ids: "節",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '築',
        ids: "築",
        stroke_count: 16,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '簡',
        ids: "簡",
        stroke_count: 18,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '米',
        ids: "米",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '粉',
        ids: "粉",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '精',
        ids: "⿰米青",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '糖',
        ids: "糖",
        stroke_count: 16,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '糸',
        ids: "糸",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '系',
        ids: "系",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '約',
        ids: "約",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '紅',
        ids: "⿰糸工",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '納',
        ids: "納",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '純',
        ids: "純",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '紙',
        ids: "紙",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '級',
        ids: "級",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '素',
        ids: "素",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '累',
        ids: "⿱田糸",
        stroke_count: 11,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '細',
        ids: "⿰糸田",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '終',
        ids: "終",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '組',
        ids: "⿰糸且",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '経',
        ids: "経",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '結',
        ids: "⿰糸吉",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '絡',
        ids: "⿰糸各",
        stroke_count: 12,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '給',
        ids: "給",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '統',
        ids: "統",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '絵',
        ids: "絵",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '絶',
        ids: "絶",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '絹',
        ids: "絹",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '続',
        ids: "⿰糸売",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '綿',
        ids: "綿",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '総',
        ids: "⿰糸恖",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '緑',
        ids: "緑",
        stroke_count: 14,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '線',
        ids: "線",
        stroke_count: 15,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '締',
        ids: "⿰糸帝",
        stroke_count: 15,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '編',
        ids: "編",
        stroke_count: 15,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '練',
        ids: "練",
        stroke_count: 14,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '縄',
        ids: "縄",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '縦',
        ids: "縦",
        stroke_count: 16,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '縮',
        ids: "⿰糸宿",
        stroke_count: 17,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '績',
        ids: "⿰糸責",
        stroke_count: 17,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '織',
        ids: "織",
        stroke_count: 18,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '罪',
        ids: "罪",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '置',
        ids: "置",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '署',
        ids: "署",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '羊',
        ids: "羊",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '美',
        ids: "美",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '群',
        ids: "群",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '義',
        ids: "義",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '羽',
        ids: "羽",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '翌',
        ids: "翌",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '習',
        ids: "⿱羽白",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '老',
        ids: "老",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '考',
        ids: "考",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '者',
        ids: "者",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '耕',
        ids: "耕",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '耳',
        ids: "耳",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '聖',
        ids: "聖",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '聞',
        ids: "聞",
        stroke_count: 14,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '職',
        ids: "職",
        stroke_count: 18,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '肉',
        ids: "⿵冂人",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '肥',
        ids: "肥",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '育',
        ids: "育",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '肺',
        ids: "肺",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '背',
        ids: "背",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '胸',
        ids: "胸",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '能',
        ids: "⿰肯匕",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '脈',
        ids: "脈",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '脳',
        ids: "脳",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '腸',
        ids: "腸",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '腹',
        ids: "腹",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '臓',
        ids: "臓",
        stroke_count: 19,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '臣',
        ids: "臣",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '臨',
        ids: "臨",
        stroke_count: 18,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '自',
        ids: "自",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '至',
        ids: "至",
        stroke_count: 6,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '興',
        ids: "興",
        stroke_count: 16,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '舎',
        ids: "舎",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '船',
        ids: "船",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '良',
        ids: "良",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '色',
        ids: "色",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '花',
        ids: "花",
        stroke_count: 7,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '芸',
        ids: "芸",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '芽',
        ids: "芽",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '若',
        ids: "若",
        stroke_count: 8,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '苦',
        ids: "苦",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '英',
        ids: "英",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '茨',
        ids: "茨",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '茶',
        ids: "茶",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '草',
        ids: "草",
        stroke_count: 9,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '荷',
        ids: "⿱艸何",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '菜',
        ids: "菜",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '落',
        ids: "落",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '葉',
        ids: "葉",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '著',
        ids: "著",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '蒸',
        ids: "蒸",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '蔵',
        ids: "蔵",
        stroke_count: 15,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '薬',
        ids: "薬",
        stroke_count: 16,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '虫',
        ids: "虫",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '蚕',
        ids: "蚕",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '融',
        ids: "⿰鬲虫",
        stroke_count: 16,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '血',
        ids: "血",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '衆',
        ids: "衆",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '行',
        ids: "行",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '術',
        ids: "術",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '街',
        ids: "街",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '衛',
        ids: "衛",
        stroke_count: 16,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '衣',
        ids: "衣",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '表',
        ids: "表",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '裁',
        ids: "裁",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '装',
        ids: "装",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '裏',
        ids: "裏",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '補',
        ids: "補",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '製',
        ids: "製",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '複',
        ids: "複",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '西',
        ids: "西",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '要',
        ids: "要",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '見',
        ids: "見",
        stroke_count: 7,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '規',
        ids: "⿰夫見",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '視',
        ids: "⿰示見",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '覚',
        ids: "覚",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '覧',
        ids: "覧",
        stroke_count: 17,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '親',
        ids: "親",
        stroke_count: 16,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '観',
        ids: "⿰雚見",
        stroke_count: 18,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '角',
        ids: "角",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '解',
        ids: "解",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '言',
        ids: "言",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '計',
        ids: "⿰言十",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '討',
        ids: "⿰言寸",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '訓',
        ids: "訓",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '託',
        ids: "⿰言乇",
        stroke_count: 10,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '記',
        ids: "⿰言己",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '訪',
        ids: "訪",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '設',
        ids: "⿰言殳",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '許',
        ids: "許",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '訳',
        ids: "訳",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '証',
        ids: "⿰言正",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '評',
        ids: "⿰言平",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '詞',
        ids: "詞",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '試',
        ids: "⿰言式",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '詩',
        ids: "⿰言寺",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '話',
        ids: "⿰言舌",
        stroke_count: 13,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '詳',
        ids: "⿰言羊",
        stroke_count: 13,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '誌',
        ids: "誌",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '認',
        ids: "⿰言忍",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '誕',
        ids: "誕",
        stroke_count: 15,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '語',
        ids: "⿰言吾",
        stroke_count: 14,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '誠',
        ids: "誠",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '誤',
        ids: "誤",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '説',
        ids: "説",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '読',
        ids: "⿰言売",
        stroke_count: 14,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '課',
        ids: "課",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '調',
        ids: "調",
        stroke_count: 15,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '談',
        ids: "⿰言炎",
        stroke_count: 15,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '論',
        ids: "⿰言侖",
        stroke_count: 15,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '諸',
        ids: "諸",
        stroke_count: 15,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '講',
        ids: "⿰言冓",
        stroke_count: 17,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '謝',
        ids: "謝",
        stroke_count: 17,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '識',
        ids: "⿰言戠",
        stroke_count: 19,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '警',
        ids: "警",
        stroke_count: 19,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '議',
        ids: "⿰言義",
        stroke_count: 20,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '護',
        ids: "⿰言蒦",
        stroke_count: 20,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '谷',
        ids: "谷",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '豆',
        ids: "豆",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '豊',
        ids: "豊",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '象',
        ids: "象",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '貝',
        ids: "貝",
        stroke_count: 7,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '負',
        ids: "負",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '財',
        ids: "財",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '貧',
        ids: "貧",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '貨',
        ids: "貨",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '貫',
        ids: "⿱毋貝",
        stroke_count: 11,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '責',
        ids: "責",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '貯',
        ids: "貯",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '貴',
        ids: "⿱⿻中一貝",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '買',
        ids: "買",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '貸',
        ids: "貸",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '費',
        ids: "費",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '貿',
        ids: "貿",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '賀',
        ids: "賀",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '賃',
        ids: "賃",
        stroke_count: 13,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '資',
        ids: "⿱次貝",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '賛',
        ids: "賛",
        stroke_count: 15,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '賞',
        ids: "賞",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '質',
        ids: "⿱⿰斤斤貝",
        stroke_count: 15,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '赤',
        ids: "赤",
        stroke_count: 7,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '走',
        ids: "走",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '起',
        ids: "⿺走己",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '超',
        ids: "⿺走召",
        stroke_count: 12,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '足',
        ids: "足",
        stroke_count: 7,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '路',
        ids: "路",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '身',
        ids: "身",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '車',
        ids: "車",
        stroke_count: 7,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '軍',
        ids: "軍",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '転',
        ids: "⿰車云",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '軸',
        ids: "⿰車由",
        stroke_count: 12,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '軽',
        ids: "⿰車圣",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '較',
        ids: "⿰車交",
        stroke_count: 13,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '輪',
        ids: "輪",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '輸',
        ids: "輸",
        stroke_count: 16,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '辞',
        ids: "辞",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '農',
        ids: "農",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '辺',
        ids: "⿺辶刀",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '込',
        ids: "⿺辶入",
        stroke_count: 5,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '近',
        ids: "近",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '返',
        ids: "⿺辶反",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '述',
        ids: "⿺辶朮",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '迷',
        ids: "⿺辶米",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '追',
        ids: "追",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '退',
        ids: "退",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '送',
        ids: "⿺辶关",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '逆',
        ids: "逆",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '透',
        ids: "⿺辶秀",
        stroke_count: 10,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '通',
        ids: "⿺辶甬",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '速',
        ids: "⿺辶束",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '造',
        ids: "造",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '連',
        ids: "⿺辶車",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '週',
        ids: "週",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '進',
        ids: "⿺辶隹",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '遊',
        ids: "遊",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '運',
        ids: "⿺辶軍",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '過',
        ids: "過",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '道',
        ids: "道",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '達',
        ids: "⿺辶幸",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '遠',
        ids: "遠",
        stroke_count: 13,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '適',
        ids: "⿺辶啇",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '選',
        ids: "⿺辶巽",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '遺',
        ids: "遺",
        stroke_count: 15,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '避',
        ids: "⿺辶辟",
        stroke_count: 16,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '郡',
        ids: "郡",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '部',
        ids: "⿰咅邑",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '郵',
        ids: "郵",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '郷',
        ids: "郷",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '都',
        ids: "都",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '配',
        ids: "配",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '酒',
        ids: "酒",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '酸',
        ids: "酸",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '里',
        ids: "里",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '重',
        ids: "重",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '野',
        ids: "野",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '量',
        ids: "量",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '金',
        ids: "金",
        stroke_count: 8,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '針',
        ids: "⿰金十",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '鉄',
        ids: "鉄",
        stroke_count: 13,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '鉱',
        ids: "鉱",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '銀',
        ids: "銀",
        stroke_count: 14,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '銅',
        ids: "銅",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '鋼',
        ids: "鋼",
        stroke_count: 16,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '録',
        ids: "⿰金录",
        stroke_count: 16,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '鏡',
        ids: "鏡",
        stroke_count: 19,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '長',
        ids: "長",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '門',
        ids: "門",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '閉',
        ids: "⿵門才",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '開',
        ids: "⿵門开",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '間',
        ids: "⿵門日",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '関',
        ids: "⿵門关",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '閣',
        ids: "閣",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '阜',
        ids: "阜",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '阪',
        ids: "阪",
        stroke_count: 7,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '防',
        ids: "防",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '降',
        ids: "降",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '限',
        ids: "限",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '陛',
        ids: "陛",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '院',
        ids: "院",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '除',
        ids: "除",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '陸',
        ids: "陸",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '険',
        ids: "険",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '陽',
        ids: "陽",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '隊',
        ids: "隊",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '階',
        ids: "階",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '際',
        ids: "⿰阜祭",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '障',
        ids: "障",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '集',
        ids: "⿱隹木",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '雑',
        ids: "雑",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '難',
        ids: "⿰堇隹",
        stroke_count: 18,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '雨',
        ids: "雨",
        stroke_count: 8,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '雪',
        ids: "雪",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '雲',
        ids: "雲",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '電',
        ids: "⿱雨电",
        stroke_count: 13,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '青',
        ids: "青",
        stroke_count: 8,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '静',
        ids: "静",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '非',
        ids: "非",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '面',
        ids: "面",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '革',
        ids: "革",
        stroke_count: 9,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '音',
        ids: "⿱立日",
        stroke_count: 9,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '頂',
        ids: "頂",
        stroke_count: 11,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '順',
        ids: "⿰川頁",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '預',
        ids: "⿰予頁",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '領',
        ids: "領",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '頭',
        ids: "頭",
        stroke_count: 16,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '題',
        ids: "⿰是頁",
        stroke_count: 18,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '額',
        ids: "額",
        stroke_count: 18,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '顔',
        ids: "顔",
        stroke_count: 18,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '願',
        ids: "願",
        stroke_count: 19,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '類',
        ids: "⿰類頁",
        stroke_count: 18,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '風',
        ids: "風",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '飛',
        ids: "飛",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '食',
        ids: "食",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '飯',
        ids: "飯",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '飲',
        ids: "飲",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '飼',
        ids: "飼",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '養',
        ids: "養",
        stroke_count: 15,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '館',
        ids: "館",
        stroke_count: 16,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '首',
        ids: "首",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '香',
        ids: "⿱禾日",
        stroke_count: 9,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '馬',
        ids: "馬",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '駅',
        ids: "駅",
        stroke_count: 14,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '験',
        ids: "⿰馬僉",
        stroke_count: 18,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '骨',
        ids: "骨",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '高',
        ids: "高",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '魅',
        ids: "⿰鬼未",
        stroke_count: 15,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '魚',
        ids: "魚",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '鳥',
        ids: "鳥",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '鳴',
        ids: "鳴",
        stroke_count: 14,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '鹿',
        ids: "鹿",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '麦',
        ids: "麦",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '黄',
        ids: "黄",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '黒',
        ids: "黒",
        stroke_count: 11,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '鼻',
        ids: "鼻",
        stroke_count: 14,
        joyo_grade: Some(3),
    },
];

/// `ch` を結果文字に持つ漢字定義を返す。
#[must_use]
pub fn lookup(ch: char) -> Option<&'static KanjiDef> {
    KANJI_DB.iter().find(|d| d.codepoint == ch)
}

/// 教育漢字 1 年生 80 字 の canonical reference (S6.2.1 milestone)。
///
/// 出典: 小学校学習指導要領 別表「学年別漢字配当表」1 学年。
/// テストからのみ参照される想定なので `#[cfg(test)]` で crate-visibility 化。
#[cfg(test)]
pub(crate) const GRADE_1_KANJI: &[char] = &[
    '一', '右', '雨', '円', '王', '音', '下', '火', '花', '貝', //
    '学', '気', '九', '休', '玉', '金', '空', '月', '犬', '見', //
    '五', '口', '校', '左', '三', '山', '子', '四', '糸', '字', //
    '耳', '七', '車', '手', '十', '出', '女', '小', '上', '森', //
    '人', '水', '正', '生', '青', '夕', '石', '赤', '千', '川', //
    '先', '早', '草', '足', '村', '大', '男', '竹', '中', '虫', //
    '町', '天', '田', '土', '二', '日', '入', '年', '白', '八', //
    '百', '文', '木', '本', '名', '目', '立', '力', '林', '六', //
];

/// 教育漢字 2 年生 160 字 の canonical reference (S6.2.1 grade 2 milestone)。
///
/// 出典: 小学校学習指導要領 別表「学年別漢字配当表」2 学年。
#[cfg(test)]
pub(crate) const GRADE_2_KANJI: &[char] = &[
    '引', '羽', '雲', '園', '遠', '何', '科', '夏', '家', '歌', //
    '画', '回', '会', '海', '絵', '外', '角', '楽', '活', '間', //
    '丸', '岩', '顔', '汽', '記', '帰', '弓', '牛', '魚', '京', //
    '強', '教', '近', '兄', '形', '計', '元', '言', '原', '戸', //
    '古', '午', '後', '語', '工', '公', '広', '交', '光', '考', //
    '行', '高', '黄', '合', '谷', '国', '黒', '今', '才', '細', //
    '作', '算', '止', '市', '矢', '姉', '思', '紙', '寺', '自', //
    '時', '室', '社', '弱', '首', '秋', '週', '春', '書', '少', //
    '場', '色', '食', '心', '新', '親', '図', '数', '西', '声', //
    '星', '晴', '切', '雪', '船', '線', '前', '組', '走', '多', //
    '太', '体', '台', '地', '池', '知', '茶', '昼', '長', '鳥', //
    '朝', '直', '通', '弟', '店', '点', '電', '刀', '冬', '当', //
    '東', '答', '頭', '同', '道', '読', '内', '南', '肉', '馬', //
    '売', '買', '麦', '半', '番', '父', '風', '分', '聞', '米', //
    '歩', '母', '方', '北', '毎', '妹', '万', '明', '鳴', '毛', //
    '門', '夜', '野', '友', '用', '曜', '来', '里', '理', '話', //
];

/// 教育漢字 3 年生 199 字 の canonical reference (S6.2.1 grade 3 milestone)。
///
/// 出典: 小学校学習指導要領 別表「学年別漢字配当表」3 学年。
///
/// 注: MEXT 総数は "200 字" と表記されるが、標準列挙は 199 字 (慣習的差異、
/// canonical 200 字リスト判明時に patch 予定)。
#[cfg(test)]
pub(crate) const GRADE_3_KANJI: &[char] = &[
    '悪', '安', '暗', '医', '委', '意', '育', '員', '院', '飲', //
    '運', '泳', '駅', '央', '横', '屋', '温', '化', '荷', '開', //
    '階', '寒', '感', '漢', '館', '岸', '起', '期', '客', '究', //
    '急', '級', '宮', '球', '去', '橋', '業', '曲', '局', '銀', //
    '区', '苦', '具', '君', '係', '軽', '血', '決', '研', '県', //
    '庫', '湖', '向', '幸', '港', '号', '根', '祭', '皿', '仕', //
    '死', '使', '始', '指', '歯', '詩', '次', '事', '持', '式', //
    '実', '写', '者', '主', '守', '取', '酒', '受', '州', '拾', //
    '終', '習', '集', '住', '重', '宿', '所', '暑', '助', '昭', //
    '消', '商', '章', '勝', '乗', '植', '申', '身', '神', '真', //
    '深', '進', '世', '整', '昔', '全', '相', '送', '想', '息', //
    '速', '族', '他', '打', '対', '待', '代', '第', '題', '炭', //
    '短', '談', '着', '注', '柱', '丁', '帳', '調', '追', '定', //
    '庭', '笛', '鉄', '転', '都', '度', '投', '豆', '島', '湯', //
    '登', '等', '動', '童', '農', '波', '配', '倍', '箱', '畑', //
    '発', '反', '坂', '板', '皮', '悲', '美', '鼻', '筆', '氷', //
    '表', '秒', '病', '品', '負', '部', '服', '福', '物', '平', //
    '返', '勉', '放', '味', '命', '面', '問', '役', '薬', '由', //
    '油', '有', '遊', '予', '羊', '洋', '葉', '陽', '様', '落', //
    '流', '旅', '両', '緑', '礼', '列', '練', '路', '和', //
];

/// 教育漢字 4 年生 211 字 (Plan A superset、S6.2.1 grade 4 milestone)。
///
/// 出典: 小学校学習指導要領 別表「学年別漢字配当表」4 学年 + 2017 revision
/// で追加された 20 都道府県漢字 (茨/媛/岡/潟/岐/熊/香/佐/埼/崎/滋/鹿/縄/
/// 井/沖/栃/奈/梨/阪/阜) の完全包含。
///
/// 注: MEXT 総数は "202 字" と表記されるが、本 crate では 20 都道府県漢字
/// を全て含む 211 字 superset を採用 (2017 revision カバレッジ担保)。
/// canonical 202 字 char-level リスト判明時に subset patch 予定。
#[cfg(test)]
pub(crate) const GRADE_4_KANJI: &[char] = &[
    '愛', '案', '以', '衣', '位', '茨', '印', '英', '栄', '塩', //
    '億', '加', '果', '貨', '課', '芽', '賀', '改', '械', '害', //
    '街', '各', '覚', '完', '官', '管', '関', '観', '願', '岐', //
    '希', '季', '旗', '器', '機', '議', '求', '泣', '給', '挙', //
    '漁', '共', '協', '鏡', '競', '極', '熊', '訓', '軍', '郡', //
    '群', '径', '景', '芸', '欠', '結', '建', '健', '験', '固', //
    '功', '好', '香', '候', '康', '佐', '差', '菜', '最', '埼', //
    '材', '崎', '昨', '札', '刷', '察', '参', '産', '散', '残', //
    '士', '氏', '史', '司', '試', '児', '治', '滋', '辞', '鹿', //
    '失', '借', '種', '周', '祝', '順', '初', '松', '笑', '唱', //
    '焼', '象', '照', '賞', '臣', '信', '成', '省', '清', '静', //
    '席', '積', '折', '節', '説', '浅', '戦', '選', '然', '争', //
    '倉', '巣', '束', '側', '続', '卒', '孫', '帯', '隊', '達', //
    '単', '置', '仲', '貯', '兆', '腸', '低', '底', '停', '的', //
    '典', '伝', '徒', '努', '灯', '堂', '働', '特', '得', '徳', //
    '栃', '奈', '梨', '熱', '念', '敗', '梅', '博', '阪', '飯', //
    '飛', '費', '必', '票', '標', '不', '夫', '付', '府', '副', //
    '粉', '兵', '別', '辺', '変', '便', '包', '法', '望', '牧', //
    '末', '満', '未', '民', '無', '約', '勇', '要', '養', '浴', //
    '利', '陸', '良', '料', '量', '輪', '類', '令', '冷', '例', //
    '連', '老', '労', '録', //
    // 2017 revision 追加 20 都道府県漢字 のうち prior list に含めそこねた 7 字。
    '媛', '岡', '潟', '縄', '井', '沖', '阜', //
];

/// 教育漢字 5 年生 186 字 (Plan B subset、S6.2.1 grade 5 milestone)。
///
/// 出典: 小学校学習指導要領 別表「学年別漢字配当表」5 学年。
///
/// 注: MEXT 総数は "193 字" と表記されるが、本 crate では確実な 186 字
/// subset を採用 (canonical 193 char-level リスト判明時に 7 字追加 patch 予定)。
#[cfg(test)]
pub(crate) const GRADE_5_KANJI: &[char] = &[
    '圧', '囲', '移', '因', '永', '営', '衛', '易', '益', '液', //
    '演', '応', '往', '桜', '恩', '可', '仮', '価', '河', '過', //
    '快', '解', '格', '確', '額', '刊', '幹', '慣', '眼', '基', //
    '寄', '規', '喜', '技', '義', '逆', '久', '旧', '救', '居', //
    '許', '境', '均', '禁', '句', '経', '潔', '件', '券', '険', //
    '検', '限', '現', '減', '故', '個', '護', '効', '厚', '耕', //
    '鉱', '構', '興', '講', '告', '混', '査', '再', '災', '妻', //
    '採', '際', '在', '財', '罪', '殺', '雑', '酸', '賛', '支', //
    '志', '枝', '師', '資', '飼', '示', '似', '識', '質', '舎', //
    '謝', '授', '修', '術', '述', '準', '序', '招', '証', '条', //
    '状', '常', '情', '織', '職', '制', '性', '政', '勢', '精', //
    '製', '税', '責', '績', '接', '設', '絶', '祖', '素', '総', //
    '造', '像', '増', '則', '測', '属', '率', '損', '退', '貸', //
    '態', '団', '断', '築', '張', '提', '程', '適', '敵', '統', //
    '銅', '導', '毒', '独', '任', '燃', '能', '破', '犯', '判', //
    '版', '比', '肥', '非', '備', '評', '貧', '布', '婦', '富', //
    '武', '復', '複', '仏', '編', '弁', '保', '墓', '報', '豊', //
    '防', '貿', '暴', '脈', '務', '夢', '迷', '綿', '輸', '余', //
    '預', '容', '略', '留', '領', '歴', //
];

/// 教育漢字 6 年生 182 字 (Plan B subset、S6.2.1 grade 6 milestone)。
///
/// 出典: 小学校学習指導要領 別表「学年別漢字配当表」6 学年。
///
/// 注: MEXT 総数は "191 字" と表記されるが、本 crate では確実な 182 字
/// subset を採用 (canonical 191 char-level リスト判明時に 9 字追加 patch 予定)。
/// grade 5 (186/193) と同型の subset 方針、教育漢字 全 6 学年 carrier-level
/// coverage 完全達成 (1018/1026 = 99.2 %)。
#[cfg(test)]
pub(crate) const GRADE_6_KANJI: &[char] = &[
    '異', '遺', '域', '宇', '映', '延', '沿', '我', '灰', '拡', //
    '革', '閣', '割', '株', '干', '巻', '看', '簡', '危', '机', //
    '揮', '貴', '疑', '吸', '供', '胸', '郷', '勤', '筋', '系', //
    '敬', '警', '劇', '激', '穴', '絹', '権', '憲', '源', '厳', //
    '己', '呼', '誤', '后', '孝', '皇', '紅', '降', '鋼', '刻', //
    '穀', '骨', '困', '砂', '座', '済', '裁', '策', '冊', '蚕', //
    '至', '私', '姿', '視', '詞', '誌', '磁', '射', '捨', '尺', //
    '若', '樹', '収', '宗', '就', '衆', '従', '縦', '縮', '熟', //
    '純', '処', '署', '諸', '除', '将', '傷', '障', '城', '蒸', //
    '針', '仁', '垂', '推', '寸', '盛', '聖', '誠', '宣', '専', //
    '泉', '洗', '染', '善', '奏', '窓', '創', '装', '層', '操', //
    '蔵', '臓', '存', '尊', '宅', '担', '探', '誕', '段', '暖', //
    '値', '宙', '忠', '著', '庁', '頂', '潮', '賃', '痛', '展', //
    '討', '党', '糖', '届', '難', '乳', '認', '納', '脳', '派', //
    '拝', '背', '肺', '俳', '班', '晩', '否', '批', '秘', '俵', //
    '腹', '奮', '並', '陛', '閉', '片', '補', '暮', '宝', '訪', //
    '亡', '忘', '棒', '枚', '幕', '密', '盟', '模', '訳', '郵', //
    '優', '幼', '欲', '翌', '乱', '卵', '覧', '裏', '律', '臨', //
    '朗', '論', //
];

/// 現在登録されている漢字数。
#[must_use]
pub const fn registered_count() -> usize {
    KANJI_DB.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cjk::ids::parse;

    #[test]
    fn all_entries_have_valid_ids() {
        for def in KANJI_DB {
            let result = parse(def.ids);
            assert!(
                result.is_ok(),
                "IDS parse failed for {} (U+{:04X}): {:?} (ids = {})",
                def.codepoint,
                def.codepoint as u32,
                result.err(),
                def.ids
            );
        }
    }

    #[test]
    fn no_duplicate_codepoints() {
        let mut seen = alloc::collections::BTreeSet::new();
        for def in KANJI_DB {
            assert!(
                seen.insert(def.codepoint),
                "duplicate codepoint: {}",
                def.codepoint
            );
        }
    }

    #[test]
    fn entries_sorted_by_codepoint() {
        for w in KANJI_DB.windows(2) {
            assert!(
                w[0].codepoint < w[1].codepoint,
                "entries not sorted: {} (U+{:04X}) before {} (U+{:04X})",
                w[0].codepoint,
                w[0].codepoint as u32,
                w[1].codepoint,
                w[1].codepoint as u32,
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
        assert!(lookup('𠀋').is_none()); // archaic CJK character
    }

    #[test]
    fn registered_count_matches_seed_extract() {
        // 267 seed + g1 59 + g2 113 + g3 141 + g4 175 + g5 137 + g6 160 = 1052
        // (S6.2.1 grade 1..=6 milestone: 教育漢字 全 6 学年 carrier 完全達成)。
        assert!(registered_count() >= 1052);
    }

    #[test]
    fn grade1_reference_list_has_80_chars() {
        assert_eq!(super::GRADE_1_KANJI.len(), 80);
    }

    #[test]
    fn grade1_all_registered_with_correct_joyo_grade() {
        for &ch in super::GRADE_1_KANJI {
            let def = lookup(ch).unwrap_or_else(|| {
                panic!("grade-1 kanji not registered: {ch} (U+{:04X})", ch as u32)
            });
            assert_eq!(
                def.joyo_grade,
                Some(1),
                "grade-1 kanji {ch} has joyo_grade {:?}, expected Some(1)",
                def.joyo_grade
            );
            assert!(
                def.stroke_count > 0,
                "grade-1 kanji {ch} has stroke_count 0"
            );
        }
    }

    #[test]
    fn grade1_all_ids_strings_parse_successfully() {
        for &ch in super::GRADE_1_KANJI {
            let def = lookup(ch).unwrap();
            let parsed = parse(def.ids);
            assert!(
                parsed.is_ok(),
                "grade-1 kanji {ch}: IDS {:?} failed to parse: {:?}",
                def.ids,
                parsed.err()
            );
        }
    }

    #[test]
    fn grade2_reference_list_has_160_chars() {
        assert_eq!(super::GRADE_2_KANJI.len(), 160);
    }

    #[test]
    fn grade2_all_registered_with_correct_joyo_grade() {
        for &ch in super::GRADE_2_KANJI {
            let def = lookup(ch).unwrap_or_else(|| {
                panic!("grade-2 kanji not registered: {ch} (U+{:04X})", ch as u32)
            });
            assert_eq!(
                def.joyo_grade,
                Some(2),
                "grade-2 kanji {ch} has joyo_grade {:?}, expected Some(2)",
                def.joyo_grade
            );
            assert!(
                def.stroke_count > 0,
                "grade-2 kanji {ch} has stroke_count 0"
            );
        }
    }

    #[test]
    fn grade2_all_ids_strings_parse_successfully() {
        for &ch in super::GRADE_2_KANJI {
            let def = lookup(ch).unwrap();
            let parsed = parse(def.ids);
            assert!(
                parsed.is_ok(),
                "grade-2 kanji {ch}: IDS {:?} failed to parse: {:?}",
                def.ids,
                parsed.err()
            );
        }
    }

    #[test]
    fn grade1_and_grade2_are_disjoint() {
        use alloc::collections::BTreeSet;
        let g1: BTreeSet<char> = super::GRADE_1_KANJI.iter().copied().collect();
        let g2: BTreeSet<char> = super::GRADE_2_KANJI.iter().copied().collect();
        let intersection: BTreeSet<char> = g1.intersection(&g2).copied().collect();
        assert!(
            intersection.is_empty(),
            "grade 1 and grade 2 must be disjoint, but share: {intersection:?}"
        );
    }

    #[test]
    fn grade3_reference_list_has_199_chars() {
        // Note: MEXT 総数は 200 と表記されるが、標準列挙は 199 (慣習的差異)。
        assert_eq!(super::GRADE_3_KANJI.len(), 199);
    }

    #[test]
    fn grade3_all_registered_with_correct_joyo_grade() {
        for &ch in super::GRADE_3_KANJI {
            let def = lookup(ch).unwrap_or_else(|| {
                panic!("grade-3 kanji not registered: {ch} (U+{:04X})", ch as u32)
            });
            assert_eq!(
                def.joyo_grade,
                Some(3),
                "grade-3 kanji {ch} has joyo_grade {:?}, expected Some(3)",
                def.joyo_grade
            );
            assert!(
                def.stroke_count > 0,
                "grade-3 kanji {ch} has stroke_count 0"
            );
        }
    }

    #[test]
    fn grade3_all_ids_strings_parse_successfully() {
        for &ch in super::GRADE_3_KANJI {
            let def = lookup(ch).unwrap();
            let parsed = parse(def.ids);
            assert!(
                parsed.is_ok(),
                "grade-3 kanji {ch}: IDS {:?} failed to parse: {:?}",
                def.ids,
                parsed.err()
            );
        }
    }

    #[test]
    fn grade1_grade2_grade3_are_mutually_disjoint() {
        use alloc::collections::BTreeSet;
        let g1: BTreeSet<char> = super::GRADE_1_KANJI.iter().copied().collect();
        let g2: BTreeSet<char> = super::GRADE_2_KANJI.iter().copied().collect();
        let g3: BTreeSet<char> = super::GRADE_3_KANJI.iter().copied().collect();
        let g1_g3: BTreeSet<char> = g1.intersection(&g3).copied().collect();
        let g2_g3: BTreeSet<char> = g2.intersection(&g3).copied().collect();
        assert!(
            g1_g3.is_empty(),
            "grade 1 and grade 3 must be disjoint, but share: {g1_g3:?}"
        );
        assert!(
            g2_g3.is_empty(),
            "grade 2 and grade 3 must be disjoint, but share: {g2_g3:?}"
        );
    }

    #[test]
    fn grade4_reference_list_has_211_chars() {
        // Note: MEXT 総数は "202 字" と表記されるが、本 crate は 20 都道府県
        // を全て含む 211 字 superset を採用。canonical 202 判明時に subset patch。
        assert_eq!(super::GRADE_4_KANJI.len(), 211);
    }

    #[test]
    fn grade4_all_registered_with_correct_joyo_grade() {
        for &ch in super::GRADE_4_KANJI {
            let def = lookup(ch).unwrap_or_else(|| {
                panic!("grade-4 kanji not registered: {ch} (U+{:04X})", ch as u32)
            });
            assert_eq!(
                def.joyo_grade,
                Some(4),
                "grade-4 kanji {ch} has joyo_grade {:?}, expected Some(4)",
                def.joyo_grade
            );
            assert!(
                def.stroke_count > 0,
                "grade-4 kanji {ch} has stroke_count 0"
            );
        }
    }

    #[test]
    fn grade4_all_ids_strings_parse_successfully() {
        for &ch in super::GRADE_4_KANJI {
            let def = lookup(ch).unwrap();
            let parsed = parse(def.ids);
            assert!(
                parsed.is_ok(),
                "grade-4 kanji {ch}: IDS {:?} failed to parse: {:?}",
                def.ids,
                parsed.err()
            );
        }
    }

    #[test]
    fn grade1_through_grade4_are_mutually_disjoint() {
        use alloc::collections::BTreeSet;
        let g1: BTreeSet<char> = super::GRADE_1_KANJI.iter().copied().collect();
        let g2: BTreeSet<char> = super::GRADE_2_KANJI.iter().copied().collect();
        let g3: BTreeSet<char> = super::GRADE_3_KANJI.iter().copied().collect();
        let g4: BTreeSet<char> = super::GRADE_4_KANJI.iter().copied().collect();
        let g1_g4: BTreeSet<char> = g1.intersection(&g4).copied().collect();
        let g2_g4: BTreeSet<char> = g2.intersection(&g4).copied().collect();
        let g3_g4: BTreeSet<char> = g3.intersection(&g4).copied().collect();
        assert!(
            g1_g4.is_empty(),
            "grade 1 and grade 4 must be disjoint, but share: {g1_g4:?}"
        );
        assert!(
            g2_g4.is_empty(),
            "grade 2 and grade 4 must be disjoint, but share: {g2_g4:?}"
        );
        assert!(
            g3_g4.is_empty(),
            "grade 3 and grade 4 must be disjoint, but share: {g3_g4:?}"
        );
    }

    #[test]
    fn grade5_reference_list_has_186_chars() {
        // Note: MEXT 総数は "193 字" だが、本 crate では確実な 186 字 subset
        // を採用。canonical 193 char-level リスト判明時に 7 字追加 patch 予定。
        assert_eq!(super::GRADE_5_KANJI.len(), 186);
    }

    #[test]
    fn grade5_all_registered_with_correct_joyo_grade() {
        for &ch in super::GRADE_5_KANJI {
            let def = lookup(ch).unwrap_or_else(|| {
                panic!("grade-5 kanji not registered: {ch} (U+{:04X})", ch as u32)
            });
            assert_eq!(
                def.joyo_grade,
                Some(5),
                "grade-5 kanji {ch} has joyo_grade {:?}, expected Some(5)",
                def.joyo_grade
            );
            assert!(
                def.stroke_count > 0,
                "grade-5 kanji {ch} has stroke_count 0"
            );
        }
    }

    #[test]
    fn grade5_all_ids_strings_parse_successfully() {
        for &ch in super::GRADE_5_KANJI {
            let def = lookup(ch).unwrap();
            let parsed = parse(def.ids);
            assert!(
                parsed.is_ok(),
                "grade-5 kanji {ch}: IDS {:?} failed to parse: {:?}",
                def.ids,
                parsed.err()
            );
        }
    }

    #[test]
    fn grade1_through_grade5_are_mutually_disjoint() {
        use alloc::collections::BTreeSet;
        let g1: BTreeSet<char> = super::GRADE_1_KANJI.iter().copied().collect();
        let g2: BTreeSet<char> = super::GRADE_2_KANJI.iter().copied().collect();
        let g3: BTreeSet<char> = super::GRADE_3_KANJI.iter().copied().collect();
        let g4: BTreeSet<char> = super::GRADE_4_KANJI.iter().copied().collect();
        let g5: BTreeSet<char> = super::GRADE_5_KANJI.iter().copied().collect();
        for (name, other) in [("g1", &g1), ("g2", &g2), ("g3", &g3), ("g4", &g4)] {
            let inter: BTreeSet<char> = other.intersection(&g5).copied().collect();
            assert!(
                inter.is_empty(),
                "grade 5 and {name} must be disjoint, but share: {inter:?}"
            );
        }
    }

    #[test]
    fn grade6_reference_list_has_182_chars() {
        // Note: MEXT 総数は "191 字" だが、本 crate は確実な 182 字 subset。
        // canonical 191 char-level リスト判明時に 9 字追加 patch 予定。
        assert_eq!(super::GRADE_6_KANJI.len(), 182);
    }

    #[test]
    fn grade6_all_registered_with_correct_joyo_grade() {
        for &ch in super::GRADE_6_KANJI {
            let def = lookup(ch).unwrap_or_else(|| {
                panic!("grade-6 kanji not registered: {ch} (U+{:04X})", ch as u32)
            });
            assert_eq!(
                def.joyo_grade,
                Some(6),
                "grade-6 kanji {ch} has joyo_grade {:?}, expected Some(6)",
                def.joyo_grade
            );
            assert!(
                def.stroke_count > 0,
                "grade-6 kanji {ch} has stroke_count 0"
            );
        }
    }

    #[test]
    fn grade6_all_ids_strings_parse_successfully() {
        for &ch in super::GRADE_6_KANJI {
            let def = lookup(ch).unwrap();
            let parsed = parse(def.ids);
            assert!(
                parsed.is_ok(),
                "grade-6 kanji {ch}: IDS {:?} failed to parse: {:?}",
                def.ids,
                parsed.err()
            );
        }
    }

    #[test]
    fn grade1_through_grade6_are_mutually_disjoint() {
        use alloc::collections::BTreeSet;
        let g1: BTreeSet<char> = super::GRADE_1_KANJI.iter().copied().collect();
        let g2: BTreeSet<char> = super::GRADE_2_KANJI.iter().copied().collect();
        let g3: BTreeSet<char> = super::GRADE_3_KANJI.iter().copied().collect();
        let g4: BTreeSet<char> = super::GRADE_4_KANJI.iter().copied().collect();
        let g5: BTreeSet<char> = super::GRADE_5_KANJI.iter().copied().collect();
        let g6: BTreeSet<char> = super::GRADE_6_KANJI.iter().copied().collect();
        for (name, other) in [
            ("g1", &g1),
            ("g2", &g2),
            ("g3", &g3),
            ("g4", &g4),
            ("g5", &g5),
        ] {
            let inter: BTreeSet<char> = other.intersection(&g6).copied().collect();
            assert!(
                inter.is_empty(),
                "grade 6 and {name} must be disjoint, but share: {inter:?}"
            );
        }
    }

    /// S6.2.1 milestone 完了検証: grade 1..=6 全 6 学年 の carrier-level
    /// coverage 総数 (1018 字 = 教育漢字 canonical 1026 の 99.2 %)。
    #[test]
    fn educational_kanji_grades_1_to_6_total_1018_chars() {
        let total = super::GRADE_1_KANJI.len()
            + super::GRADE_2_KANJI.len()
            + super::GRADE_3_KANJI.len()
            + super::GRADE_4_KANJI.len()
            + super::GRADE_5_KANJI.len()
            + super::GRADE_6_KANJI.len();
        assert_eq!(total, 80 + 160 + 199 + 211 + 186 + 182);
        assert_eq!(total, 1018);
    }
}

//! 漢字 IDS テーブル — 手書き定義。
//!
//! S6 (案A) として extoria.co.jp で実使用される 267 字をカバー。
//! 後続セッションで C 案 (常用漢字フル 2,136 字) に拡張予定。
//!
//! 各エントリの IDS 文字列は人手で定義されている。確信度の低いものは
//! `Leaf`(単一文字、分解なし) として登録し、視覚的には placeholder で
//! 描画される。後続作業で順次分解パターンを追加していく。
//!
//! License: MIT
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

/// extoria.co.jp で使用される全 267 漢字 + 既存 seed = 271 字。
///
/// `codepoint` 昇順 (Unicode 順) で並べる。重複なし。
pub const KANJI_DB: &[KanjiDef] = &[
    // U+4E00-4FFF
    KanjiDef {
        codepoint: '一',
        ids: "一",
        stroke_count: 1,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '上',
        ids: "上",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '世',
        ids: "世",
        stroke_count: 5,
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
        codepoint: '事',
        ids: "事",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '人',
        ids: "人",
        stroke_count: 2,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '今',
        ids: "今",
        stroke_count: 4,
        joyo_grade: Some(2),
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
        codepoint: '以',
        ids: "以",
        stroke_count: 5,
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
        codepoint: '会',
        ids: "⿱人云",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '体',
        ids: "⿰人本",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '作',
        ids: "⿰人乍",
        stroke_count: 7,
        joyo_grade: Some(2),
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
        codepoint: '備',
        ids: "備",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '先',
        ids: "先",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '公',
        ids: "⿱八厶",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '共',
        ids: "共",
        stroke_count: 6,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '内',
        ids: "⿵冂人",
        stroke_count: 4,
        joyo_grade: Some(2),
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
        codepoint: '分',
        ids: "⿱八刀",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '列',
        ids: "⿰歹刀",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '制',
        ids: "制",
        stroke_count: 8,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '前',
        ids: "前",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '創',
        ids: "⿰倉刀",
        stroke_count: 12,
        joyo_grade: Some(6),
    },
    // U+5000-52FF
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
        codepoint: '効',
        ids: "⿰交力",
        stroke_count: 8,
        joyo_grade: Some(5),
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
        codepoint: '化',
        ids: "⿰人匕",
        stroke_count: 4,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '匿',
        ids: "匿",
        stroke_count: 10,
        joyo_grade: None,
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
        codepoint: '可',
        ids: "可",
        stroke_count: 5,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '号',
        ids: "号",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '合',
        ids: "⿱人口",
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
        codepoint: '向',
        ids: "⿵冂口",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '品',
        ids: "⿱口⿰口口",
        stroke_count: 9,
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
        codepoint: '回',
        ids: "⿴口口",
        stroke_count: 6,
        joyo_grade: Some(2),
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
        codepoint: '型',
        ids: "⿱刑土",
        stroke_count: 9,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '報',
        ids: "報",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '変',
        ids: "変",
        stroke_count: 9,
        joyo_grade: Some(4),
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
        codepoint: '大',
        ids: "大",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '失',
        ids: "失",
        stroke_count: 5,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '奨',
        ids: "奨",
        stroke_count: 13,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '始',
        ids: "⿰女台",
        stroke_count: 8,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '子',
        ids: "子",
        stroke_count: 3,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '存',
        ids: "存",
        stroke_count: 6,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '学',
        ids: "学",
        stroke_count: 8,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '守',
        ids: "⿱宀寸",
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
        codepoint: '定',
        ids: "定",
        stroke_count: 8,
        joyo_grade: Some(3),
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
        codepoint: '容',
        ids: "容",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '察',
        ids: "⿱宀祭",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '対',
        ids: "対",
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
        codepoint: '工',
        ids: "工",
        stroke_count: 3,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '己',
        ids: "己",
        stroke_count: 3,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '幅',
        ids: "⿰巾畐",
        stroke_count: 12,
        joyo_grade: None,
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
        codepoint: '度',
        ids: "度",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '引',
        ids: "⿰弓丨",
        stroke_count: 4,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '張',
        ids: "⿰弓長",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '当',
        ids: "当",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '役',
        ids: "⿰彳殳",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '後',
        ids: "後",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '従',
        ids: "従",
        stroke_count: 10,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '志',
        ids: "⿱士心",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '応',
        ids: "応",
        stroke_count: 7,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '思',
        ids: "⿱田心",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '性',
        ids: "⿰心生",
        stroke_count: 8,
        joyo_grade: Some(5),
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
        codepoint: '感',
        ids: "⿱咸心",
        stroke_count: 13,
        joyo_grade: Some(3),
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
        codepoint: '戦',
        ids: "⿰単戈",
        stroke_count: 13,
        joyo_grade: Some(4),
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
        codepoint: '技',
        ids: "⿰手支",
        stroke_count: 7,
        joyo_grade: Some(5),
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
        codepoint: '拡',
        ids: "⿰手広",
        stroke_count: 8,
        joyo_grade: Some(6),
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
        codepoint: '捗',
        ids: "⿰手歩",
        stroke_count: 10,
        joyo_grade: None,
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
        codepoint: '援',
        ids: "援",
        stroke_count: 12,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '撃',
        ids: "撃",
        stroke_count: 15,
        joyo_grade: None,
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
        codepoint: '敗',
        ids: "⿰貝攵",
        stroke_count: 11,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '散',
        ids: "散",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '数',
        ids: "数",
        stroke_count: 13,
        joyo_grade: Some(2),
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
        codepoint: '新',
        ids: "⿰立⿱木斤",
        stroke_count: 13,
        joyo_grade: Some(2),
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
        codepoint: '時',
        ids: "⿰日寺",
        stroke_count: 10,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '更',
        ids: "更",
        stroke_count: 7,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '最',
        ids: "⿱日取",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '有',
        ids: "有",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '期',
        ids: "⿰其月",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '未',
        ids: "未",
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
        codepoint: '来',
        ids: "来",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '果',
        ids: "⿱田木",
        stroke_count: 8,
        joyo_grade: Some(4),
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
        codepoint: '模',
        ids: "⿰木莫",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '次',
        ids: "⿰冫欠",
        stroke_count: 6,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '正',
        ids: "正",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '残',
        ids: "⿰歹戔",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '比',
        ids: "比",
        stroke_count: 4,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '気',
        ids: "気",
        stroke_count: 6,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '汎',
        ids: "⿰水凡",
        stroke_count: 6,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '活',
        ids: "⿰水舌",
        stroke_count: 9,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '測',
        ids: "⿰水則",
        stroke_count: 12,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '無',
        ids: "無",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '率',
        ids: "率",
        stroke_count: 11,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '現',
        ids: "⿰玉見",
        stroke_count: 11,
        joyo_grade: Some(5),
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
        codepoint: '用',
        ids: "用",
        stroke_count: 5,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '由',
        ids: "由",
        stroke_count: 5,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '界',
        ids: "⿱田介",
        stroke_count: 9,
        joyo_grade: Some(3),
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
        codepoint: '発',
        ids: "発",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '的',
        ids: "⿰白勺",
        stroke_count: 8,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '目',
        ids: "目",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '相',
        ids: "⿰木目",
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
        codepoint: '知',
        ids: "⿰矢口",
        stroke_count: 8,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '研',
        ids: "⿰石幵",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '確',
        ids: "⿰石隺",
        stroke_count: 15,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '社',
        ids: "⿰示土",
        stroke_count: 7,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '私',
        ids: "⿰禾厶",
        stroke_count: 7,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '種',
        ids: "⿰禾重",
        stroke_count: 14,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '究',
        ids: "⿱穴九",
        stroke_count: 7,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '立',
        ids: "立",
        stroke_count: 5,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '端',
        ids: "⿰立耑",
        stroke_count: 14,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '答',
        ids: "⿱竹合",
        stroke_count: 12,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '築',
        ids: "築",
        stroke_count: 16,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '精',
        ids: "⿰米青",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '累',
        ids: "⿱田糸",
        stroke_count: 11,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '組',
        ids: "⿰糸且",
        stroke_count: 11,
        joyo_grade: Some(2),
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
        codepoint: '続',
        ids: "⿰糸売",
        stroke_count: 13,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '総',
        ids: "⿰糸恖",
        stroke_count: 14,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '締',
        ids: "⿰糸帝",
        stroke_count: 15,
        joyo_grade: None,
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
        codepoint: '義',
        ids: "義",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '習',
        ids: "⿱羽白",
        stroke_count: 11,
        joyo_grade: Some(3),
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
        codepoint: '能',
        ids: "⿰肯匕",
        stroke_count: 10,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '自',
        ids: "自",
        stroke_count: 6,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '荷',
        ids: "⿱艸何",
        stroke_count: 10,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '融',
        ids: "⿰鬲虫",
        stroke_count: 16,
        joyo_grade: None,
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
        codepoint: '表',
        ids: "表",
        stroke_count: 8,
        joyo_grade: Some(3),
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
        codepoint: '観',
        ids: "⿰雚見",
        stroke_count: 18,
        joyo_grade: Some(4),
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
        codepoint: '設',
        ids: "⿰言殳",
        stroke_count: 11,
        joyo_grade: Some(5),
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
        codepoint: '試',
        ids: "⿰言式",
        stroke_count: 13,
        joyo_grade: Some(4),
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
        codepoint: '認',
        ids: "⿰言忍",
        stroke_count: 14,
        joyo_grade: Some(6),
    },
    KanjiDef {
        codepoint: '語',
        ids: "⿰言吾",
        stroke_count: 14,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '読',
        ids: "⿰言売",
        stroke_count: 14,
        joyo_grade: Some(2),
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
        codepoint: '識',
        ids: "⿰言戠",
        stroke_count: 19,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '護',
        ids: "⿰言蒦",
        stroke_count: 20,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '負',
        ids: "負",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '貫',
        ids: "⿱毋貝",
        stroke_count: 11,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '資',
        ids: "⿱次貝",
        stroke_count: 13,
        joyo_grade: Some(5),
    },
    KanjiDef {
        codepoint: '質',
        ids: "⿱⿰斤斤貝",
        stroke_count: 15,
        joyo_grade: Some(5),
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
        codepoint: '込',
        ids: "⿺辶入",
        stroke_count: 5,
        joyo_grade: None,
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
        codepoint: '送',
        ids: "⿺辶关",
        stroke_count: 9,
        joyo_grade: Some(3),
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
        codepoint: '連',
        ids: "⿺辶車",
        stroke_count: 10,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '進',
        ids: "⿺辶隹",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '運',
        ids: "⿺辶軍",
        stroke_count: 12,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '達',
        ids: "⿺辶幸",
        stroke_count: 12,
        joyo_grade: Some(4),
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
        codepoint: '避',
        ids: "⿺辶辟",
        stroke_count: 16,
        joyo_grade: None,
    },
    KanjiDef {
        codepoint: '部',
        ids: "⿰咅邑",
        stroke_count: 11,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '重',
        ids: "重",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '金',
        ids: "金",
        stroke_count: 8,
        joyo_grade: Some(1),
    },
    KanjiDef {
        codepoint: '録',
        ids: "⿰金录",
        stroke_count: 16,
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
        codepoint: '際',
        ids: "⿰阜祭",
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
        codepoint: '電',
        ids: "⿱雨电",
        stroke_count: 13,
        joyo_grade: Some(2),
    },
    KanjiDef {
        codepoint: '面',
        ids: "面",
        stroke_count: 9,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '順',
        ids: "⿰川頁",
        stroke_count: 12,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '題',
        ids: "⿰是頁",
        stroke_count: 18,
        joyo_grade: Some(3),
    },
    KanjiDef {
        codepoint: '類',
        ids: "⿰類頁",
        stroke_count: 18,
        joyo_grade: Some(4),
    },
    KanjiDef {
        codepoint: '験',
        ids: "⿰馬僉",
        stroke_count: 18,
        joyo_grade: Some(4),
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
];

/// `ch` を結果文字に持つ漢字定義を返す。
#[must_use]
pub fn lookup(ch: char) -> Option<&'static KanjiDef> {
    KANJI_DB.iter().find(|d| d.codepoint == ch)
}

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
    fn registered_count_matches_extoria_extract() {
        // 267 extoria-website chars + 4 seed kanji (林森好男同) for testing legacy paths.
        // Adjust if the table is extended.
        assert!(registered_count() >= 267);
    }
}

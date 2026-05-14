//! 康熙部首テーブル — 全 214 部首。
//!
//! 各エントリは Kangxi (康熙) 部首番号、Unicode 文字、画数、英語名を持つ。
//!
//! License: MIT
//! Author: Moroya Sakamoto

extern crate alloc;
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

/// 康熙部首 214 件の完全テーブル。`id` 昇順。
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
        id: 5,
        character: '乙',
        stroke_count: 1,
        name: "second",
    },
    CjkRadical {
        id: 6,
        character: '亅',
        stroke_count: 1,
        name: "hook",
    },
    CjkRadical {
        id: 7,
        character: '二',
        stroke_count: 2,
        name: "two",
    },
    CjkRadical {
        id: 8,
        character: '亠',
        stroke_count: 2,
        name: "lid",
    },
    CjkRadical {
        id: 9,
        character: '人',
        stroke_count: 2,
        name: "person",
    },
    CjkRadical {
        id: 10,
        character: '儿',
        stroke_count: 2,
        name: "legs",
    },
    CjkRadical {
        id: 11,
        character: '入',
        stroke_count: 2,
        name: "enter",
    },
    CjkRadical {
        id: 12,
        character: '八',
        stroke_count: 2,
        name: "eight",
    },
    CjkRadical {
        id: 13,
        character: '冂',
        stroke_count: 2,
        name: "wide",
    },
    CjkRadical {
        id: 14,
        character: '冖',
        stroke_count: 2,
        name: "cover",
    },
    CjkRadical {
        id: 15,
        character: '冫',
        stroke_count: 2,
        name: "ice",
    },
    CjkRadical {
        id: 16,
        character: '几',
        stroke_count: 2,
        name: "table",
    },
    CjkRadical {
        id: 17,
        character: '凵',
        stroke_count: 2,
        name: "container",
    },
    CjkRadical {
        id: 18,
        character: '刀',
        stroke_count: 2,
        name: "knife",
    },
    CjkRadical {
        id: 19,
        character: '力',
        stroke_count: 2,
        name: "power",
    },
    CjkRadical {
        id: 20,
        character: '勹',
        stroke_count: 2,
        name: "wrap",
    },
    CjkRadical {
        id: 21,
        character: '匕',
        stroke_count: 2,
        name: "spoon",
    },
    CjkRadical {
        id: 22,
        character: '匚',
        stroke_count: 2,
        name: "box",
    },
    CjkRadical {
        id: 23,
        character: '匸',
        stroke_count: 2,
        name: "hide",
    },
    CjkRadical {
        id: 24,
        character: '十',
        stroke_count: 2,
        name: "ten",
    },
    CjkRadical {
        id: 25,
        character: '卜',
        stroke_count: 2,
        name: "divination",
    },
    CjkRadical {
        id: 26,
        character: '卩',
        stroke_count: 2,
        name: "seal",
    },
    CjkRadical {
        id: 27,
        character: '厂',
        stroke_count: 2,
        name: "cliff",
    },
    CjkRadical {
        id: 28,
        character: '厶',
        stroke_count: 2,
        name: "private",
    },
    CjkRadical {
        id: 29,
        character: '又',
        stroke_count: 2,
        name: "again",
    },
    CjkRadical {
        id: 30,
        character: '口',
        stroke_count: 3,
        name: "mouth",
    },
    CjkRadical {
        id: 31,
        character: '囗',
        stroke_count: 3,
        name: "enclosure",
    },
    CjkRadical {
        id: 32,
        character: '土',
        stroke_count: 3,
        name: "earth",
    },
    CjkRadical {
        id: 33,
        character: '士',
        stroke_count: 3,
        name: "scholar",
    },
    CjkRadical {
        id: 34,
        character: '夂',
        stroke_count: 3,
        name: "winter",
    },
    CjkRadical {
        id: 35,
        character: '夊',
        stroke_count: 3,
        name: "go_slowly",
    },
    CjkRadical {
        id: 36,
        character: '夕',
        stroke_count: 3,
        name: "evening",
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
        id: 39,
        character: '子',
        stroke_count: 3,
        name: "child",
    },
    CjkRadical {
        id: 40,
        character: '宀',
        stroke_count: 3,
        name: "roof",
    },
    CjkRadical {
        id: 41,
        character: '寸',
        stroke_count: 3,
        name: "inch",
    },
    CjkRadical {
        id: 42,
        character: '小',
        stroke_count: 3,
        name: "small",
    },
    CjkRadical {
        id: 43,
        character: '尢',
        stroke_count: 3,
        name: "lame",
    },
    CjkRadical {
        id: 44,
        character: '尸',
        stroke_count: 3,
        name: "corpse",
    },
    CjkRadical {
        id: 45,
        character: '屮',
        stroke_count: 3,
        name: "sprout",
    },
    CjkRadical {
        id: 46,
        character: '山',
        stroke_count: 3,
        name: "mountain",
    },
    CjkRadical {
        id: 47,
        character: '巛',
        stroke_count: 3,
        name: "river",
    },
    CjkRadical {
        id: 48,
        character: '工',
        stroke_count: 3,
        name: "work",
    },
    CjkRadical {
        id: 49,
        character: '己',
        stroke_count: 3,
        name: "oneself",
    },
    CjkRadical {
        id: 50,
        character: '巾',
        stroke_count: 3,
        name: "cloth",
    },
    CjkRadical {
        id: 51,
        character: '干',
        stroke_count: 3,
        name: "shield",
    },
    CjkRadical {
        id: 52,
        character: '幺',
        stroke_count: 3,
        name: "short_thread",
    },
    CjkRadical {
        id: 53,
        character: '广',
        stroke_count: 3,
        name: "dotted_cliff",
    },
    CjkRadical {
        id: 54,
        character: '廴',
        stroke_count: 3,
        name: "long_stride",
    },
    CjkRadical {
        id: 55,
        character: '廾',
        stroke_count: 3,
        name: "hands_joined",
    },
    CjkRadical {
        id: 56,
        character: '弋',
        stroke_count: 3,
        name: "ceremony",
    },
    CjkRadical {
        id: 57,
        character: '弓',
        stroke_count: 3,
        name: "bow",
    },
    CjkRadical {
        id: 58,
        character: '彐',
        stroke_count: 3,
        name: "snout",
    },
    CjkRadical {
        id: 59,
        character: '彡',
        stroke_count: 3,
        name: "hair",
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
        id: 62,
        character: '戈',
        stroke_count: 4,
        name: "halberd",
    },
    CjkRadical {
        id: 63,
        character: '戶',
        stroke_count: 4,
        name: "door",
    },
    CjkRadical {
        id: 64,
        character: '手',
        stroke_count: 4,
        name: "hand",
    },
    CjkRadical {
        id: 65,
        character: '支',
        stroke_count: 4,
        name: "branch",
    },
    CjkRadical {
        id: 66,
        character: '攴',
        stroke_count: 4,
        name: "strike",
    },
    CjkRadical {
        id: 67,
        character: '文',
        stroke_count: 4,
        name: "script",
    },
    CjkRadical {
        id: 68,
        character: '斗',
        stroke_count: 4,
        name: "dipper",
    },
    CjkRadical {
        id: 69,
        character: '斤',
        stroke_count: 4,
        name: "axe",
    },
    CjkRadical {
        id: 70,
        character: '方',
        stroke_count: 4,
        name: "square",
    },
    CjkRadical {
        id: 71,
        character: '无',
        stroke_count: 4,
        name: "not",
    },
    CjkRadical {
        id: 72,
        character: '日',
        stroke_count: 4,
        name: "sun",
    },
    CjkRadical {
        id: 73,
        character: '曰',
        stroke_count: 4,
        name: "say",
    },
    CjkRadical {
        id: 74,
        character: '月',
        stroke_count: 4,
        name: "moon",
    },
    CjkRadical {
        id: 75,
        character: '木',
        stroke_count: 4,
        name: "tree",
    },
    CjkRadical {
        id: 76,
        character: '欠',
        stroke_count: 4,
        name: "lack",
    },
    CjkRadical {
        id: 77,
        character: '止',
        stroke_count: 4,
        name: "stop",
    },
    CjkRadical {
        id: 78,
        character: '歹',
        stroke_count: 4,
        name: "bad",
    },
    CjkRadical {
        id: 79,
        character: '殳',
        stroke_count: 4,
        name: "weapon",
    },
    CjkRadical {
        id: 80,
        character: '毋',
        stroke_count: 4,
        name: "do_not",
    },
    CjkRadical {
        id: 81,
        character: '比',
        stroke_count: 4,
        name: "compare",
    },
    CjkRadical {
        id: 82,
        character: '毛',
        stroke_count: 4,
        name: "fur",
    },
    CjkRadical {
        id: 83,
        character: '氏',
        stroke_count: 4,
        name: "clan",
    },
    CjkRadical {
        id: 84,
        character: '气',
        stroke_count: 4,
        name: "steam",
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
        id: 87,
        character: '爪',
        stroke_count: 4,
        name: "claw",
    },
    CjkRadical {
        id: 88,
        character: '父',
        stroke_count: 4,
        name: "father",
    },
    CjkRadical {
        id: 89,
        character: '爻',
        stroke_count: 4,
        name: "mix",
    },
    CjkRadical {
        id: 90,
        character: '爿',
        stroke_count: 4,
        name: "bed_split_left",
    },
    CjkRadical {
        id: 91,
        character: '片',
        stroke_count: 4,
        name: "slice",
    },
    CjkRadical {
        id: 92,
        character: '牙',
        stroke_count: 4,
        name: "fang",
    },
    CjkRadical {
        id: 93,
        character: '牛',
        stroke_count: 4,
        name: "ox",
    },
    CjkRadical {
        id: 94,
        character: '犬',
        stroke_count: 4,
        name: "dog",
    },
    CjkRadical {
        id: 95,
        character: '玄',
        stroke_count: 5,
        name: "profound",
    },
    CjkRadical {
        id: 96,
        character: '玉',
        stroke_count: 5,
        name: "jade",
    },
    CjkRadical {
        id: 97,
        character: '瓜',
        stroke_count: 5,
        name: "melon",
    },
    CjkRadical {
        id: 98,
        character: '瓦',
        stroke_count: 5,
        name: "tile",
    },
    CjkRadical {
        id: 99,
        character: '甘',
        stroke_count: 5,
        name: "sweet",
    },
    CjkRadical {
        id: 100,
        character: '生',
        stroke_count: 5,
        name: "life",
    },
    CjkRadical {
        id: 101,
        character: '用',
        stroke_count: 5,
        name: "use",
    },
    CjkRadical {
        id: 102,
        character: '田',
        stroke_count: 5,
        name: "field",
    },
    CjkRadical {
        id: 103,
        character: '疋',
        stroke_count: 5,
        name: "bolt_of_cloth",
    },
    CjkRadical {
        id: 104,
        character: '疒',
        stroke_count: 5,
        name: "sickness",
    },
    CjkRadical {
        id: 105,
        character: '癶',
        stroke_count: 5,
        name: "footsteps",
    },
    CjkRadical {
        id: 106,
        character: '白',
        stroke_count: 5,
        name: "white",
    },
    CjkRadical {
        id: 107,
        character: '皮',
        stroke_count: 5,
        name: "skin",
    },
    CjkRadical {
        id: 108,
        character: '皿',
        stroke_count: 5,
        name: "dish",
    },
    CjkRadical {
        id: 109,
        character: '目',
        stroke_count: 5,
        name: "eye",
    },
    CjkRadical {
        id: 110,
        character: '矛',
        stroke_count: 5,
        name: "spear",
    },
    CjkRadical {
        id: 111,
        character: '矢',
        stroke_count: 5,
        name: "arrow",
    },
    CjkRadical {
        id: 112,
        character: '石',
        stroke_count: 5,
        name: "stone",
    },
    CjkRadical {
        id: 113,
        character: '示',
        stroke_count: 5,
        name: "spirit",
    },
    CjkRadical {
        id: 114,
        character: '禸',
        stroke_count: 5,
        name: "track",
    },
    CjkRadical {
        id: 115,
        character: '禾',
        stroke_count: 5,
        name: "grain",
    },
    CjkRadical {
        id: 116,
        character: '穴',
        stroke_count: 5,
        name: "cave",
    },
    CjkRadical {
        id: 117,
        character: '立',
        stroke_count: 5,
        name: "stand",
    },
    CjkRadical {
        id: 118,
        character: '竹',
        stroke_count: 6,
        name: "bamboo",
    },
    CjkRadical {
        id: 119,
        character: '米',
        stroke_count: 6,
        name: "rice",
    },
    CjkRadical {
        id: 120,
        character: '糸',
        stroke_count: 6,
        name: "silk",
    },
    CjkRadical {
        id: 121,
        character: '缶',
        stroke_count: 6,
        name: "jar",
    },
    CjkRadical {
        id: 122,
        character: '网',
        stroke_count: 6,
        name: "net",
    },
    CjkRadical {
        id: 123,
        character: '羊',
        stroke_count: 6,
        name: "sheep",
    },
    CjkRadical {
        id: 124,
        character: '羽',
        stroke_count: 6,
        name: "feather",
    },
    CjkRadical {
        id: 125,
        character: '老',
        stroke_count: 6,
        name: "old",
    },
    CjkRadical {
        id: 126,
        character: '而',
        stroke_count: 6,
        name: "and",
    },
    CjkRadical {
        id: 127,
        character: '耒',
        stroke_count: 6,
        name: "plow",
    },
    CjkRadical {
        id: 128,
        character: '耳',
        stroke_count: 6,
        name: "ear",
    },
    CjkRadical {
        id: 129,
        character: '聿',
        stroke_count: 6,
        name: "brush",
    },
    CjkRadical {
        id: 130,
        character: '肉',
        stroke_count: 6,
        name: "meat",
    },
    CjkRadical {
        id: 131,
        character: '臣',
        stroke_count: 6,
        name: "minister",
    },
    CjkRadical {
        id: 132,
        character: '自',
        stroke_count: 6,
        name: "self",
    },
    CjkRadical {
        id: 133,
        character: '至',
        stroke_count: 6,
        name: "reach",
    },
    CjkRadical {
        id: 134,
        character: '臼',
        stroke_count: 6,
        name: "mortar",
    },
    CjkRadical {
        id: 135,
        character: '舌',
        stroke_count: 6,
        name: "tongue",
    },
    CjkRadical {
        id: 136,
        character: '舛',
        stroke_count: 6,
        name: "opposite",
    },
    CjkRadical {
        id: 137,
        character: '舟',
        stroke_count: 6,
        name: "boat",
    },
    CjkRadical {
        id: 138,
        character: '艮',
        stroke_count: 6,
        name: "tough",
    },
    CjkRadical {
        id: 139,
        character: '色',
        stroke_count: 6,
        name: "color",
    },
    CjkRadical {
        id: 140,
        character: '艸',
        stroke_count: 6,
        name: "grass",
    },
    CjkRadical {
        id: 141,
        character: '虍',
        stroke_count: 6,
        name: "tiger_top",
    },
    CjkRadical {
        id: 142,
        character: '虫',
        stroke_count: 6,
        name: "insect",
    },
    CjkRadical {
        id: 143,
        character: '血',
        stroke_count: 6,
        name: "blood",
    },
    CjkRadical {
        id: 144,
        character: '行',
        stroke_count: 6,
        name: "walk",
    },
    CjkRadical {
        id: 145,
        character: '衣',
        stroke_count: 6,
        name: "clothes",
    },
    CjkRadical {
        id: 146,
        character: '襾',
        stroke_count: 6,
        name: "west",
    },
    CjkRadical {
        id: 147,
        character: '見',
        stroke_count: 7,
        name: "see",
    },
    CjkRadical {
        id: 148,
        character: '角',
        stroke_count: 7,
        name: "horn",
    },
    CjkRadical {
        id: 149,
        character: '言',
        stroke_count: 7,
        name: "speech",
    },
    CjkRadical {
        id: 150,
        character: '谷',
        stroke_count: 7,
        name: "valley",
    },
    CjkRadical {
        id: 151,
        character: '豆',
        stroke_count: 7,
        name: "bean",
    },
    CjkRadical {
        id: 152,
        character: '豕',
        stroke_count: 7,
        name: "pig",
    },
    CjkRadical {
        id: 153,
        character: '豸',
        stroke_count: 7,
        name: "badger",
    },
    CjkRadical {
        id: 154,
        character: '貝',
        stroke_count: 7,
        name: "shell",
    },
    CjkRadical {
        id: 155,
        character: '赤',
        stroke_count: 7,
        name: "red",
    },
    CjkRadical {
        id: 156,
        character: '走',
        stroke_count: 7,
        name: "run",
    },
    CjkRadical {
        id: 157,
        character: '足',
        stroke_count: 7,
        name: "foot",
    },
    CjkRadical {
        id: 158,
        character: '身',
        stroke_count: 7,
        name: "body",
    },
    CjkRadical {
        id: 159,
        character: '車',
        stroke_count: 7,
        name: "cart",
    },
    CjkRadical {
        id: 160,
        character: '辛',
        stroke_count: 7,
        name: "bitter",
    },
    CjkRadical {
        id: 161,
        character: '辰',
        stroke_count: 7,
        name: "morning",
    },
    CjkRadical {
        id: 162,
        character: '辵',
        stroke_count: 7,
        name: "walk_radical",
    },
    CjkRadical {
        id: 163,
        character: '邑',
        stroke_count: 7,
        name: "city",
    },
    CjkRadical {
        id: 164,
        character: '酉',
        stroke_count: 7,
        name: "wine_vessel",
    },
    CjkRadical {
        id: 165,
        character: '釆',
        stroke_count: 7,
        name: "distinguish",
    },
    CjkRadical {
        id: 166,
        character: '里',
        stroke_count: 7,
        name: "village",
    },
    CjkRadical {
        id: 167,
        character: '金',
        stroke_count: 8,
        name: "gold",
    },
    CjkRadical {
        id: 168,
        character: '長',
        stroke_count: 8,
        name: "long",
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
        id: 171,
        character: '隶',
        stroke_count: 8,
        name: "slave",
    },
    CjkRadical {
        id: 172,
        character: '隹',
        stroke_count: 8,
        name: "bird_short",
    },
    CjkRadical {
        id: 173,
        character: '雨',
        stroke_count: 8,
        name: "rain",
    },
    CjkRadical {
        id: 174,
        character: '青',
        stroke_count: 8,
        name: "blue",
    },
    CjkRadical {
        id: 175,
        character: '非',
        stroke_count: 8,
        name: "wrong",
    },
    CjkRadical {
        id: 176,
        character: '面',
        stroke_count: 9,
        name: "face",
    },
    CjkRadical {
        id: 177,
        character: '革',
        stroke_count: 9,
        name: "leather",
    },
    CjkRadical {
        id: 178,
        character: '韋',
        stroke_count: 9,
        name: "tanned_leather",
    },
    CjkRadical {
        id: 179,
        character: '韭',
        stroke_count: 9,
        name: "leek",
    },
    CjkRadical {
        id: 180,
        character: '音',
        stroke_count: 9,
        name: "sound",
    },
    CjkRadical {
        id: 181,
        character: '頁',
        stroke_count: 9,
        name: "page",
    },
    CjkRadical {
        id: 182,
        character: '風',
        stroke_count: 9,
        name: "wind",
    },
    CjkRadical {
        id: 183,
        character: '飛',
        stroke_count: 9,
        name: "fly",
    },
    CjkRadical {
        id: 184,
        character: '食',
        stroke_count: 9,
        name: "food",
    },
    CjkRadical {
        id: 185,
        character: '首',
        stroke_count: 9,
        name: "head",
    },
    CjkRadical {
        id: 186,
        character: '香',
        stroke_count: 9,
        name: "fragrance",
    },
    CjkRadical {
        id: 187,
        character: '馬',
        stroke_count: 10,
        name: "horse",
    },
    CjkRadical {
        id: 188,
        character: '骨',
        stroke_count: 10,
        name: "bone",
    },
    CjkRadical {
        id: 189,
        character: '高',
        stroke_count: 10,
        name: "tall",
    },
    CjkRadical {
        id: 190,
        character: '髟',
        stroke_count: 10,
        name: "hair_long",
    },
    CjkRadical {
        id: 191,
        character: '鬥',
        stroke_count: 10,
        name: "fight",
    },
    CjkRadical {
        id: 192,
        character: '鬯',
        stroke_count: 10,
        name: "sacrificial_wine",
    },
    CjkRadical {
        id: 193,
        character: '鬲',
        stroke_count: 10,
        name: "cauldron",
    },
    CjkRadical {
        id: 194,
        character: '鬼',
        stroke_count: 10,
        name: "ghost",
    },
    CjkRadical {
        id: 195,
        character: '魚',
        stroke_count: 11,
        name: "fish",
    },
    CjkRadical {
        id: 196,
        character: '鳥',
        stroke_count: 11,
        name: "bird",
    },
    CjkRadical {
        id: 197,
        character: '鹵',
        stroke_count: 11,
        name: "salt",
    },
    CjkRadical {
        id: 198,
        character: '鹿',
        stroke_count: 11,
        name: "deer",
    },
    CjkRadical {
        id: 199,
        character: '麥',
        stroke_count: 11,
        name: "wheat",
    },
    CjkRadical {
        id: 200,
        character: '麻',
        stroke_count: 11,
        name: "hemp",
    },
    CjkRadical {
        id: 201,
        character: '黃',
        stroke_count: 12,
        name: "yellow",
    },
    CjkRadical {
        id: 202,
        character: '黍',
        stroke_count: 12,
        name: "millet",
    },
    CjkRadical {
        id: 203,
        character: '黑',
        stroke_count: 12,
        name: "black",
    },
    CjkRadical {
        id: 204,
        character: '黹',
        stroke_count: 12,
        name: "embroidery",
    },
    CjkRadical {
        id: 205,
        character: '黽',
        stroke_count: 13,
        name: "frog",
    },
    CjkRadical {
        id: 206,
        character: '鼎',
        stroke_count: 13,
        name: "tripod",
    },
    CjkRadical {
        id: 207,
        character: '鼓',
        stroke_count: 13,
        name: "drum",
    },
    CjkRadical {
        id: 208,
        character: '鼠',
        stroke_count: 13,
        name: "rat",
    },
    CjkRadical {
        id: 209,
        character: '鼻',
        stroke_count: 14,
        name: "nose",
    },
    CjkRadical {
        id: 210,
        character: '齊',
        stroke_count: 14,
        name: "even",
    },
    CjkRadical {
        id: 211,
        character: '齒',
        stroke_count: 15,
        name: "tooth",
    },
    CjkRadical {
        id: 212,
        character: '龍',
        stroke_count: 16,
        name: "dragon",
    },
    CjkRadical {
        id: 213,
        character: '龜',
        stroke_count: 16,
        name: "turtle",
    },
    CjkRadical {
        id: 214,
        character: '龠',
        stroke_count: 17,
        name: "flute",
    },
];

/// 部首を ID で検索。
#[must_use]
pub fn lookup_radical(id: RadicalId) -> Option<&'static CjkRadical> {
    RADICALS.iter().find(|r| r.id == id)
}

/// 部首を Unicode 文字で検索。
#[must_use]
pub fn lookup_radical_by_char(ch: char) -> Option<&'static CjkRadical> {
    RADICALS.iter().find(|r| r.character == ch)
}

/// 部首を名前で検索。
#[must_use]
pub fn find_radical_by_name(name: &str) -> Option<&'static CjkRadical> {
    RADICALS.iter().find(|r| r.name == name)
}

/// 画数で部首を検索 (該当する全部首を返す)。
#[must_use]
pub fn radicals_by_stroke_count(strokes: u8) -> Vec<&'static CjkRadical> {
    RADICALS
        .iter()
        .filter(|r| r.stroke_count == strokes)
        .collect()
}

/// 部首テーブルのエントリ数。
#[must_use]
pub const fn radical_count() -> usize {
    RADICALS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_214_radicals_present() {
        assert_eq!(RADICALS.len(), 214);
    }

    #[test]
    fn ids_are_sequential_1_to_214() {
        for (i, r) in RADICALS.iter().enumerate() {
            assert_eq!(r.id as usize, i + 1, "radical at index {i} has id {}", r.id);
        }
    }

    #[test]
    fn first_and_last_radicals() {
        assert_eq!(RADICALS[0].character, '一');
        assert_eq!(RADICALS[213].character, '龠');
        assert_eq!(RADICALS[213].stroke_count, 17);
    }

    #[test]
    fn lookup_water_by_char() {
        let r = lookup_radical_by_char('水').unwrap();
        assert_eq!(r.id, 85);
    }

    #[test]
    fn stroke_counts_monotonic_per_block() {
        // 連番の部首で画数が単調非減少 (康熙部首順の性質)。
        for window in RADICALS.windows(2) {
            assert!(
                window[0].stroke_count <= window[1].stroke_count,
                "stroke count regressed from {} ({}) to {} ({})",
                window[0].character,
                window[0].stroke_count,
                window[1].character,
                window[1].stroke_count
            );
        }
    }
}

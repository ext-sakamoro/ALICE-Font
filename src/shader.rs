//! Fragment-shader analytical path — pack glyphs as GPU-friendly uniforms.
//!
//! S6.3.1 milestone: ROADMAP core value「データを送るな、法則を送れ」
//! の直接実証。CPU rasterisation を bypass し、cubic Bezier control points
//! と variable-width pen envelope を uniform として GPU に送り、fragment
//! shader が per-pixel に SDF を解析評価する。
//!
//! Payload には bitmap / atlas / texture data は一切含まれない。
//! runtime texture upload = 0 byte を [`AnalyticalGlyphPayload::payload_bytes`]
//! で数値保証。
//!
//! ## 使い方
//!
//! ```
//! use alice_font::{param::MetaFontParams, shader::pack_glyph_for_shader};
//!
//! let params = MetaFontParams::sans_regular();
//! let payload = pack_glyph_for_shader('森', &params)
//!     .expect("森 must be in KANJI_DB");
//!
//! // strokes を uniform として GLSL に送信
//! assert!(payload.strokes.len() > 0);
//! assert!(payload.payload_bytes() < 1024);
//! ```
//!
//! ## GLSL 側の対応 shader
//!
//! `shaders/analytical_glyph.frag.glsl` を参照。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec::Vec;

use crate::glyph::kanji::build_skeleton;
use crate::param::MetaFontParams;
use crate::stroke::PenModel;

/// GPU shader に送る、単一 kanji の全 stroke + pen パラメータ。
///
/// Bitmap や atlas は含まない。all-analytical evaluation の入力データのみ。
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AnalyticalGlyphPayload {
    /// 全 stroke の cubic Bezier control points + per-endpoint pen 半幅。
    pub strokes: Vec<AnalyticalStroke>,
    /// Pen model パラメータ (fragment shader で contrast / roundness 参照用)。
    pub pen: AnalyticalPen,
    /// Advance width (em unit)。text layout 側で参照。
    pub advance: f32,
}

impl AnalyticalGlyphPayload {
    /// Payload の実 byte size (Vec header 分は除外、shader に送る値の総 byte)。
    ///
    /// stroke 1 個 = 10 f32 (4 control points × 2 + 2 pen 半幅) = 40 byte。
    /// pen = 4 f32 = 16 byte。advance = 4 byte。合計 = `strokes * 40 + 20` byte。
    #[must_use]
    pub fn payload_bytes(&self) -> usize {
        self.strokes.len() * core::mem::size_of::<AnalyticalStroke>()
            + core::mem::size_of::<AnalyticalPen>()
            + core::mem::size_of::<f32>()
    }

    /// Stroke 数を返す (GLSL 側 uStrokeCount uniform 用)。
    #[must_use]
    pub fn stroke_count(&self) -> usize {
        self.strokes.len()
    }
}

/// 単一 cubic Bezier stroke の shader-ready 表現。
///
/// GLSL 側では 5 vec4 (`p0.xy p1.xy p2.xy p3.xy widthStart widthEnd`) と
/// してレイアウトされる想定 (`shaders/analytical_glyph.frag.glsl` 参照)。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(C)]
pub struct AnalyticalStroke {
    /// Bezier 制御点 0 (始点)、em 座標系 (0..1)。
    pub p0: [f32; 2],
    /// Bezier 制御点 1、em 座標系。
    pub p1: [f32; 2],
    /// Bezier 制御点 2、em 座標系。
    pub p2: [f32; 2],
    /// Bezier 制御点 3 (終点)、em 座標系。
    pub p3: [f32; 2],
    /// t=0 (始点) での pen 半幅 (em 単位)。
    pub half_width_start: f32,
    /// t=1 (終点) での pen 半幅 (em 単位)。
    pub half_width_end: f32,
}

/// Pen モデルパラメータ (fragment shader が anti-aliasing / contrast 用に参照)。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(C)]
pub struct AnalyticalPen {
    /// 基準 pen 半幅 (em 単位)。
    pub base_width: f32,
    /// Contrast 比率 (0 = monowidth、1 = high contrast)。
    pub contrast: f32,
    /// Pen 角度 (radians)。
    pub pen_angle: f32,
    /// Corner roundness。
    pub roundness: f32,
}

/// Kanji を fragment-shader-ready payload に pack する。
///
/// [`crate::glyph::kanji::build_skeleton`] 経由で IDS 展開された Bezier
/// stroke を取得し、per-stroke に endpoint pen 半幅を計算して詰める。
///
/// # Returns
/// - `Some(payload)` — kanji が KANJI_DB に登録済 + stroke 1 個以上
/// - `None` — 未登録 or IDS parse 失敗 or stroke 0
#[must_use]
pub fn pack_glyph_for_shader(ch: char, params: &MetaFontParams) -> Option<AnalyticalGlyphPayload> {
    let skel = build_skeleton(ch)?;
    let pen_model = PenModel::from_params(params);

    let mut strokes = Vec::with_capacity(skel.stroke_count);
    for i in 0..skel.stroke_count {
        let s = &skel.strokes[i];
        // t=0 / t=1 での tangent 方向から pen 半幅を算出。
        // Cubic Bezier 数値安定性: t=0 の tangent = 3(p1 - p0)、
        // t=1 の tangent = 3(p3 - p2)。degenerate line (p0==p1) の場合は
        // p3 - p0 で fallback して pen 半幅を base_width に落とす。
        let t0 = s.tangent(0.0);
        let t1 = s.tangent(1.0);
        strokes.push(AnalyticalStroke {
            p0: [s.p0.x, s.p0.y],
            p1: [s.p1.x, s.p1.y],
            p2: [s.p2.x, s.p2.y],
            p3: [s.p3.x, s.p3.y],
            half_width_start: pen_model.half_width(t0),
            half_width_end: pen_model.half_width(t1),
        });
    }

    Some(AnalyticalGlyphPayload {
        strokes,
        pen: AnalyticalPen {
            base_width: pen_model.base_width,
            contrast: pen_model.contrast,
            pen_angle: pen_model.pen_angle,
            roundness: pen_model.roundness,
        },
        advance: skel.advance,
    })
}

// ============================================================================
// S6.3.2 — Multi-kanji text rendering payload
// ============================================================================

/// 単一 glyph の text layout 内配置情報 (em 座標)。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(C)]
pub struct GlyphLayout {
    /// この glyph の左端 x 座標 (em 単位、text 全体の 0 起点)。
    pub x_offset: f32,
    /// この glyph の advance (em 単位、通常 pack 時 skel.advance と一致)。
    pub advance: f32,
}

/// 複数 kanji を text として render するための payload (S6.3.2 sub-goal 2)。
///
/// Glyphs are packed in reading order (`text.chars()`)。skip された文字
/// (未登録 kanji、hiragana / katakana / ASCII) は `layout[i].x_offset` +
/// `layout[i].advance` で空白を占有するが `glyphs[i]` は空 (未登録扱い)。
///
/// 現行実装: 全 glyph が KANJI_DB 登録済 + build_skeleton 成功のみ pack、
/// 未登録は Layout も出力しない (`total_advance` 計算対象外)。
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MultiGlyphPayload {
    /// 各 kanji の shader-ready payload (登録済のみ、reading order)。
    pub glyphs: Vec<AnalyticalGlyphPayload>,
    /// 各 glyph の text layout 内配置 (glyphs と同 length + 同 order)。
    pub layout: Vec<GlyphLayout>,
    /// Text 全体の advance (em 単位、= 全 glyph.advance の総和)。
    pub total_advance: f32,
}

impl MultiGlyphPayload {
    /// 全 glyph の payload_bytes 合計 (bitmap / atlas 抜き、shader 送信 byte)。
    #[must_use]
    pub fn payload_bytes(&self) -> usize {
        self.glyphs
            .iter()
            .map(AnalyticalGlyphPayload::payload_bytes)
            .sum::<usize>()
            + self.layout.len() * core::mem::size_of::<GlyphLayout>()
            + core::mem::size_of::<f32>()
    }

    /// Glyph 数を返す (`text.chars().count()` と一致するとは限らない、
    /// 未登録 skip の可能性あり)。
    #[must_use]
    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }
}

/// Text (複数 kanji) を fragment-shader-ready `MultiGlyphPayload` に pack。
///
/// - reading order で `text.chars()` を舐め、[`pack_glyph_for_shader`] で
///   pack 成功したものだけ `glyphs` に追加。
/// - `x_offset` は「pack 済 glyph advance の累積」で計算 (skip した文字の
///   分は空白として現れない、text は詰まった layout になる)。
/// - text が空 or 全 char が pack 失敗の場合は `None` を返す。
///
/// # Examples
///
/// ```
/// use alice_font::{param::MetaFontParams, shader::pack_string_for_shader};
///
/// let params = MetaFontParams::sans_regular();
/// let text = pack_string_for_shader("森明", &params).unwrap();
/// assert_eq!(text.glyph_count(), 2);
/// assert!(text.total_advance > 0.0);
/// assert_eq!(text.layout[0].x_offset, 0.0);
/// assert!(text.layout[1].x_offset > 0.0);
/// ```
#[must_use]
pub fn pack_string_for_shader(text: &str, params: &MetaFontParams) -> Option<MultiGlyphPayload> {
    let mut glyphs = Vec::new();
    let mut layout = Vec::new();
    let mut cursor = 0.0f32;
    for ch in text.chars() {
        if let Some(payload) = pack_glyph_for_shader(ch, params) {
            let advance = payload.advance;
            layout.push(GlyphLayout {
                x_offset: cursor,
                advance,
            });
            cursor += advance;
            glyphs.push(payload);
        }
    }
    if glyphs.is_empty() {
        None
    } else {
        Some(MultiGlyphPayload {
            glyphs,
            layout,
            total_advance: cursor,
        })
    }
}

// ============================================================================
// S6.3.2b — std140-compatible UBO serialization (fixed layout + full GLSL 統合)
// ============================================================================

/// GLSL `layout(std140) uniform GlyphBlock` に対応する fixed-size UBO 総 byte 数。
///
/// = MAX_STROKES (16) × [`AnalyticalStroke::STD140_SIZE`] (48)
/// + [`AnalyticalPen::STD140_SIZE`] (16) + `uMeta` vec4 (16) = **800 byte**。
///
/// UBO は shader 側の GlyphBlock struct と bit-exact 一致する必要あり
/// (`shaders/analytical_glyph.frag.glsl` v3 と一致確認)。
pub const GLYPH_UBO_SIZE: usize =
    MAX_STROKES * AnalyticalStroke::STD140_SIZE + AnalyticalPen::STD140_SIZE + 16;

/// UBO 内 stroke array の最大要素数 ([`crate::glyph::MAX_GLYPH_STROKES`] と一致)。
pub const MAX_STROKES: usize = crate::glyph::MAX_GLYPH_STROKES;

/// GLSL std140 UBO layout の byte-level serialization API (S6.3.2b)。
///
/// OpenGL / WebGL2 UBO の `layout(std140) uniform GlyphBlock` にそのまま
/// upload できる 800 byte fixed size の byte 列を生成する。shader は
/// `shaders/analytical_glyph.frag.glsl` v3 を参照。
///
/// # std140 packing rule (vec4-grouped Stroke struct 採用理由)
///
/// std140 は array of struct の場合、struct 全体を vec4 alignment (16 byte) に
/// padding するため、`vec2 p0/p1/p2/p3 + float w0/w1` を平置きすると
/// 40 byte 実質 + 8 byte trailing pad = 48 byte / element となる。
///
/// これを明示的な vec4 grouping (`vec4 p01 / vec4 p23 / vec4 widths`) に
/// することで pad なしに 48 byte を占め、shader 側の struct declaration も
/// simple になる (`vec4` 3 個で完結、alignment 一致)。
impl AnalyticalStroke {
    /// std140 layout での 1 stroke の byte size (vec4-grouped struct、pad なし)。
    ///
    /// - `p01 = vec4(p0.x, p0.y, p1.x, p1.y)` = 16B
    /// - `p23 = vec4(p2.x, p2.y, p3.x, p3.y)` = 16B
    /// - `widths = vec4(half_width_start, half_width_end, 0, 0)` = 16B
    /// - 合計 = 48 byte / stroke、GLSL `Stroke { vec4 p01; vec4 p23; vec4 widths; }` と一致。
    pub const STD140_SIZE: usize = 48;

    /// std140 準拠 (vec4-grouped) の byte 列を出力 (little-endian、`Vec` に append)。
    pub fn write_std140_bytes(&self, out: &mut Vec<u8>) {
        // p01 = vec4(p0.x, p0.y, p1.x, p1.y) — 16B
        out.extend_from_slice(&self.p0[0].to_le_bytes());
        out.extend_from_slice(&self.p0[1].to_le_bytes());
        out.extend_from_slice(&self.p1[0].to_le_bytes());
        out.extend_from_slice(&self.p1[1].to_le_bytes());
        // p23 = vec4(p2.x, p2.y, p3.x, p3.y) — 16B
        out.extend_from_slice(&self.p2[0].to_le_bytes());
        out.extend_from_slice(&self.p2[1].to_le_bytes());
        out.extend_from_slice(&self.p3[0].to_le_bytes());
        out.extend_from_slice(&self.p3[1].to_le_bytes());
        // widths = vec4(w_start, w_end, 0, 0) — 16B
        out.extend_from_slice(&self.half_width_start.to_le_bytes());
        out.extend_from_slice(&self.half_width_end.to_le_bytes());
        out.extend_from_slice(&0f32.to_le_bytes());
        out.extend_from_slice(&0f32.to_le_bytes());
    }
}

impl AnalyticalPen {
    /// std140 layout での pen struct 総 byte size (vec4 相当、pad なし)。
    ///
    /// 4 float × 4B = 16B、GLSL `uniform vec4 uPen` と直接一致。
    pub const STD140_SIZE: usize = 16;

    /// std140 準拠 byte 列を出力 (little-endian、vec4 相当)。
    pub fn write_std140_bytes(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.base_width.to_le_bytes());
        out.extend_from_slice(&self.contrast.to_le_bytes());
        out.extend_from_slice(&self.pen_angle.to_le_bytes());
        out.extend_from_slice(&self.roundness.to_le_bytes());
    }
}

impl AnalyticalGlyphPayload {
    /// std140 UBO 用の fixed byte size ([`GLYPH_UBO_SIZE`] = 800B、stroke 数によらず)。
    ///
    /// GLSL UBO は静的サイズ、実 stroke 数が MAX_STROKES 未満でも 800B 消費。
    /// 未使用 stroke slot は zero-fill、shader は `uMeta.y` (stroke_count) で
    /// 有効 slot 数を判定して skip する。
    #[must_use]
    pub fn std140_size(&self) -> usize {
        GLYPH_UBO_SIZE
    }

    /// std140 UBO 用の byte 列を出力 (little-endian、800B fixed)。
    ///
    /// Layout (`shaders/analytical_glyph.frag.glsl` v3 と bit-exact 一致):
    /// - `Stroke uStrokes[16]` (48B × 16 = 768B)
    ///   - 実 stroke 数分書き込み、残りは zero-fill
    /// - `vec4 uPen` (16B): base_width, contrast, pen_angle, roundness
    /// - `vec4 uMeta` (16B): advance, stroke_count_as_f32, 0, 0
    /// - **合計 = 800 byte**
    #[must_use]
    pub fn to_std140_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(GLYPH_UBO_SIZE);
        // Stroke array (実 stroke 数)
        for stroke in &self.strokes {
            stroke.write_std140_bytes(&mut out);
        }
        // Padding to MAX_STROKES (16) — 未使用 slot は zero-fill
        let padded_slots = MAX_STROKES.saturating_sub(self.strokes.len());
        for _ in 0..padded_slots {
            out.extend_from_slice(&[0u8; AnalyticalStroke::STD140_SIZE]);
        }
        // Pen (vec4)
        self.pen.write_std140_bytes(&mut out);
        // uMeta = vec4(advance, stroke_count as f32, 0, 0)
        out.extend_from_slice(&self.advance.to_le_bytes());
        #[allow(clippy::cast_precision_loss)]
        let stroke_count_f = self.strokes.len() as f32;
        out.extend_from_slice(&stroke_count_f.to_le_bytes());
        out.extend_from_slice(&0f32.to_le_bytes());
        out.extend_from_slice(&0f32.to_le_bytes());
        debug_assert_eq!(out.len(), GLYPH_UBO_SIZE);
        out
    }
}

// ============================================================================
// S6.3.2b.4 — Flat multi-glyph UBO (dynamic 1-level indexing、GPU compat 向上)
// ============================================================================

/// 1 chunk (= 1 `drawArraysInstanced` call) に pack できる最大 glyph 数。
///
/// WebGL2 `GL_MAX_UNIFORM_BLOCK_SIZE` 最低保証 16 KB 内に fit する上限。
/// Flat layout: 12,288B strokes + 512B metas + 16B uTextMeta = 12,816B。
pub const MAX_GLYPHS_PER_DRAW: usize = 16;

/// GLSL `layout(std140) uniform MultiGlyphBlock` に対応する fixed-size UBO 総 byte 数。
///
/// **v0.5.4 (S6.3.2b.4) flat layout**:
/// - `Stroke uStrokes[256]` (16 glyph × 16 stroke = 256 slot × 48 = 12,288B)
/// - `GlyphMeta uMetas[16]` (16 × 32 = 512B、pen + meta / glyph)
/// - `vec4 uTextMeta` (16B、total_advance + chunk_glyph_count_f + 0 + 0)
/// - **合計 = 12,816 byte** (v0.5.3 と同 size、内部レイアウトのみ変更)
///
/// **変更理由**: v0.5.3 nested `uGlyphs[i].strokes[j]` の 2 段 dynamic indexing
/// が一部 GPU で不動作 (`WebGL2 std140` spec 上 legal だが実装依存)。
/// Flat `uStrokes[i * 16 + j]` の 1 段 dynamic indexing に置換して portability
/// 向上、multi-glyph 描画の readability 復活 (user report 2026-07-30)。
pub const MULTI_GLYPH_UBO_SIZE: usize =
    MULTI_GLYPH_STROKE_AREA_SIZE + MULTI_GLYPH_METAS_AREA_SIZE + 16;

/// UBO 内 stroke array の総サイズ (16 glyph × 16 stroke × 48 = 12,288B)。
pub const MULTI_GLYPH_STROKE_AREA_SIZE: usize =
    MAX_GLYPHS_PER_DRAW * MAX_STROKES * AnalyticalStroke::STD140_SIZE;

/// UBO 内 GlyphMeta array の総サイズ (16 × 32 = 512B、pen 16B + meta 16B / glyph)。
pub const MULTI_GLYPH_METAS_AREA_SIZE: usize = MAX_GLYPHS_PER_DRAW * 32;

impl MultiGlyphPayload {
    /// 1 chunk = ≤ [`MAX_GLYPHS_PER_DRAW`] glyphs、text 全体を chunk 分割した数。
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.glyphs.len().div_ceil(MAX_GLYPHS_PER_DRAW)
    }

    /// Multi-glyph UBO byte chunks を生成 (each chunk = 12,816 byte fixed size)。
    ///
    /// **v0.5.4 flat layout** (`shaders/analytical_multi_glyph.frag.glsl` v3.2 と一致):
    /// - Offset 0..12,288: `Stroke uStrokes[256]` (glyph 0 stroke 0..15、glyph 1
    ///   stroke 0..15、... 、未使用 slot は zero-fill)
    /// - Offset 12,288..12,800: `GlyphMeta uMetas[16]` (16 × 32、per-glyph
    ///   `{ vec4 pen; vec4 meta; }` where meta = `(advance, stroke_count_f,
    ///   x_offset, 0)`)
    /// - Offset 12,800..12,816: `vec4 uTextMeta` (total_advance, chunk_glyph_count_f, 0, 0)
    ///
    /// v0.5.3 との差: nested struct 廃止、flat array で 1 段 dynamic indexing に。
    #[must_use]
    pub fn to_std140_ubo_chunks(&self) -> Vec<Vec<u8>> {
        let n_chunks = self.chunk_count();
        let mut chunks = Vec::with_capacity(n_chunks);
        for chunk_idx in 0..n_chunks {
            let start = chunk_idx * MAX_GLYPHS_PER_DRAW;
            let end = (start + MAX_GLYPHS_PER_DRAW).min(self.glyphs.len());
            let chunk_glyph_count = end - start;
            let mut out = Vec::with_capacity(MULTI_GLYPH_UBO_SIZE);

            // ---------- Section 1: Stroke uStrokes[256] (12,288B) ----------
            for i in start..end {
                let glyph = &self.glyphs[i];
                for stroke in &glyph.strokes {
                    stroke.write_std140_bytes(&mut out);
                }
                let padded = MAX_STROKES.saturating_sub(glyph.strokes.len());
                for _ in 0..padded {
                    out.extend_from_slice(&[0u8; AnalyticalStroke::STD140_SIZE]);
                }
            }
            let padded_glyph_slots = MAX_GLYPHS_PER_DRAW - chunk_glyph_count;
            for _ in 0..padded_glyph_slots {
                out.extend_from_slice(&[0u8; MAX_STROKES * AnalyticalStroke::STD140_SIZE]);
            }
            debug_assert_eq!(out.len(), MULTI_GLYPH_STROKE_AREA_SIZE);

            // ---------- Section 2: GlyphMeta uMetas[16] (512B) ----------
            // 各 GlyphMeta = { vec4 pen; vec4 meta; } = 32B (std140 pad なし)
            for i in start..end {
                let glyph = &self.glyphs[i];
                let x_offset = self.layout[i].x_offset;
                glyph.pen.write_std140_bytes(&mut out); // 16B
                                                        // meta = vec4(advance, stroke_count_f, x_offset, 0)
                out.extend_from_slice(&glyph.advance.to_le_bytes());
                #[allow(clippy::cast_precision_loss)]
                let stroke_count_f = glyph.strokes.len() as f32;
                out.extend_from_slice(&stroke_count_f.to_le_bytes());
                out.extend_from_slice(&x_offset.to_le_bytes());
                out.extend_from_slice(&0f32.to_le_bytes());
            }
            // Padding for unused glyph metas (32B × unused)
            for _ in 0..padded_glyph_slots {
                out.extend_from_slice(&[0u8; 32]);
            }
            debug_assert_eq!(
                out.len(),
                MULTI_GLYPH_STROKE_AREA_SIZE + MULTI_GLYPH_METAS_AREA_SIZE
            );

            // ---------- Section 3: uTextMeta vec4 (16B) ----------
            out.extend_from_slice(&self.total_advance.to_le_bytes());
            #[allow(clippy::cast_precision_loss)]
            let count_f = chunk_glyph_count as f32;
            out.extend_from_slice(&count_f.to_le_bytes());
            out.extend_from_slice(&0f32.to_le_bytes());
            out.extend_from_slice(&0f32.to_le_bytes());

            debug_assert_eq!(out.len(), MULTI_GLYPH_UBO_SIZE);
            chunks.push(out);
        }
        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cjk::ids_db::GRADE_1_KANJI;

    #[test]
    fn pack_mori_returns_payload() {
        let params = MetaFontParams::sans_regular();
        let payload = pack_glyph_for_shader('森', &params).unwrap();
        assert!(payload.stroke_count() > 0, "森 must have strokes");
        assert!(payload.pen.base_width > 0.0);
    }

    #[test]
    fn pack_mei_returns_payload() {
        let params = MetaFontParams::sans_regular();
        let payload = pack_glyph_for_shader('明', &params).unwrap();
        assert!(payload.stroke_count() > 0);
    }

    #[test]
    fn pack_unregistered_returns_none() {
        let params = MetaFontParams::sans_regular();
        // U+9FA5 = 龥 (unregistered per existing kanji.rs test)。
        assert!(pack_glyph_for_shader('龥', &params).is_none());
    }

    #[test]
    fn payload_size_below_1kb_for_mori_and_mei() {
        let params = MetaFontParams::sans_regular();
        for ch in ['森', '明'] {
            let payload = pack_glyph_for_shader(ch, &params).unwrap();
            let bytes = payload.payload_bytes();
            assert!(
                bytes < 1024,
                "{ch}: payload {bytes} byte >= 1 KB (想定を超えた stroke count)"
            );
        }
    }

    #[test]
    fn pack_grade1_all_80_chars_no_panic() {
        let params = MetaFontParams::sans_regular();
        for &ch in GRADE_1_KANJI {
            // Some grade-1 chars are Leaf → renders placeholder rectangle
            // (4 strokes), still packable. All 80 must return Some.
            let payload = pack_glyph_for_shader(ch, &params)
                .unwrap_or_else(|| panic!("grade-1 {ch} (U+{:04X}) failed to pack", ch as u32));
            assert!(
                payload.stroke_count() > 0,
                "grade-1 {ch}: pack succeeded but stroke_count == 0"
            );
        }
    }

    #[test]
    fn stroke_layout_is_c_repr_40_bytes() {
        // GLSL 側 uniform packing 前提: 10 f32 = 40 byte を維持。
        assert_eq!(core::mem::size_of::<AnalyticalStroke>(), 40);
        assert_eq!(core::mem::size_of::<AnalyticalPen>(), 16);
    }

    #[test]
    fn pack_preserves_stroke_control_points_from_skeleton() {
        let params = MetaFontParams::sans_regular();
        let payload = pack_glyph_for_shader('森', &params).unwrap();
        let skel = build_skeleton('森').unwrap();
        assert_eq!(payload.stroke_count(), skel.stroke_count);
        for (i, packed) in payload.strokes.iter().enumerate() {
            let s = &skel.strokes[i];
            assert_eq!(packed.p0, [s.p0.x, s.p0.y]);
            assert_eq!(packed.p1, [s.p1.x, s.p1.y]);
            assert_eq!(packed.p2, [s.p2.x, s.p2.y]);
            assert_eq!(packed.p3, [s.p3.x, s.p3.y]);
            assert!(packed.half_width_start > 0.0);
            assert!(packed.half_width_end > 0.0);
        }
    }

    // ------------------------------------------------------------------------
    // S6.3.2 sub-goal 2: MultiGlyphPayload + pack_string_for_shader tests
    // ------------------------------------------------------------------------

    #[test]
    fn pack_string_empty_returns_none() {
        let params = MetaFontParams::sans_regular();
        assert!(pack_string_for_shader("", &params).is_none());
    }

    #[test]
    fn pack_string_all_unregistered_returns_none() {
        let params = MetaFontParams::sans_regular();
        // U+9FA5 龥 は kanji.rs test で確認済の未登録字。
        assert!(pack_string_for_shader("龥", &params).is_none());
    }

    #[test]
    fn pack_string_single_glyph_matches_single_pack() {
        let params = MetaFontParams::sans_regular();
        let single = pack_glyph_for_shader('森', &params).unwrap();
        let multi = pack_string_for_shader("森", &params).unwrap();
        assert_eq!(multi.glyph_count(), 1);
        assert_eq!(multi.glyphs[0], single);
        assert_eq!(multi.layout[0].x_offset, 0.0);
        assert_eq!(multi.layout[0].advance, single.advance);
        assert_eq!(multi.total_advance, single.advance);
    }

    #[test]
    fn pack_string_multi_glyph_layout_is_cumulative() {
        let params = MetaFontParams::sans_regular();
        let text = pack_string_for_shader("森明林", &params).unwrap();
        assert_eq!(text.glyph_count(), 3);
        assert_eq!(text.layout.len(), 3);
        // x_offset は累積、advance は per-glyph。
        assert_eq!(text.layout[0].x_offset, 0.0);
        assert_eq!(
            text.layout[1].x_offset, text.layout[0].advance,
            "layout[1].x_offset must == layout[0].advance"
        );
        assert_eq!(
            text.layout[2].x_offset,
            text.layout[0].advance + text.layout[1].advance
        );
        // total_advance = 全 advance の和。
        let sum: f32 = text.layout.iter().map(|g| g.advance).sum();
        assert!(
            (text.total_advance - sum).abs() < 1e-6,
            "total_advance {} != sum {}",
            text.total_advance,
            sum
        );
    }

    #[test]
    fn pack_string_skips_unregistered_glyphs() {
        let params = MetaFontParams::sans_regular();
        // "森龥明" — 龥 は未登録、skip されて 2 glyph の text になる。
        let text = pack_string_for_shader("森龥明", &params).unwrap();
        assert_eq!(text.glyph_count(), 2);
        // 森 + 明 の順序が保持されている。
        assert!(text.glyphs[0].stroke_count() > 0);
        assert!(text.glyphs[1].stroke_count() > 0);
    }

    #[test]
    fn multi_glyph_payload_bytes_matches_sum() {
        let params = MetaFontParams::sans_regular();
        let text = pack_string_for_shader("森明", &params).unwrap();
        let expected = text
            .glyphs
            .iter()
            .map(AnalyticalGlyphPayload::payload_bytes)
            .sum::<usize>()
            + text.layout.len() * core::mem::size_of::<GlyphLayout>()
            + core::mem::size_of::<f32>();
        assert_eq!(text.payload_bytes(), expected);
    }

    // ------------------------------------------------------------------------
    // S6.3.2 sub-goal 1: to_std140_bytes serialization tests
    // ------------------------------------------------------------------------

    #[test]
    fn std140_stroke_size_is_48_bytes() {
        // v0.5.2 (S6.3.2b) fix: 56B → 48B (vec4-grouped struct、std140 spec 準拠)。
        assert_eq!(AnalyticalStroke::STD140_SIZE, 48);
        let stroke = AnalyticalStroke {
            p0: [0.1, 0.2],
            p1: [0.3, 0.4],
            p2: [0.5, 0.6],
            p3: [0.7, 0.8],
            half_width_start: 0.05,
            half_width_end: 0.07,
        };
        let mut buf = Vec::new();
        stroke.write_std140_bytes(&mut buf);
        assert_eq!(buf.len(), AnalyticalStroke::STD140_SIZE);
    }

    #[test]
    fn std140_pen_size_is_16_bytes() {
        assert_eq!(AnalyticalPen::STD140_SIZE, 16);
        let pen = AnalyticalPen {
            base_width: 0.03,
            contrast: 0.1,
            pen_angle: 0.5,
            roundness: 0.3,
        };
        let mut buf = Vec::new();
        pen.write_std140_bytes(&mut buf);
        assert_eq!(buf.len(), AnalyticalPen::STD140_SIZE);
    }

    #[test]
    fn std140_glyph_payload_size_is_fixed_800_bytes() {
        // v0.5.2 (S6.3.2b): UBO は静的 800B、stroke 数によらず一定。
        assert_eq!(GLYPH_UBO_SIZE, MAX_STROKES * 48 + 16 + 16);
        assert_eq!(GLYPH_UBO_SIZE, 800);
        let params = MetaFontParams::sans_regular();
        for ch in ['森', '明', '一'] {
            let payload = pack_glyph_for_shader(ch, &params).unwrap();
            assert_eq!(payload.std140_size(), GLYPH_UBO_SIZE);
            assert_eq!(payload.to_std140_bytes().len(), GLYPH_UBO_SIZE);
        }
    }

    #[test]
    fn std140_stroke_layout_byte_level_verification() {
        // v0.5.2 vec4-grouped layout (std140 spec 準拠):
        // - p01 = vec4(p0.x, p0.y, p1.x, p1.y) — offset 0..16
        // - p23 = vec4(p2.x, p2.y, p3.x, p3.y) — offset 16..32
        // - widths = vec4(w_start, w_end, 0, 0) — offset 32..48
        let stroke = AnalyticalStroke {
            p0: [1.0, 2.0],
            p1: [3.0, 4.0],
            p2: [5.0, 6.0],
            p3: [7.0, 8.0],
            half_width_start: 0.5,
            half_width_end: 0.75,
        };
        let mut buf = Vec::new();
        stroke.write_std140_bytes(&mut buf);
        assert_eq!(buf.len(), 48);
        // p01 = vec4
        assert_eq!(&buf[0..4], &1.0f32.to_le_bytes());
        assert_eq!(&buf[4..8], &2.0f32.to_le_bytes());
        assert_eq!(&buf[8..12], &3.0f32.to_le_bytes());
        assert_eq!(&buf[12..16], &4.0f32.to_le_bytes());
        // p23 = vec4
        assert_eq!(&buf[16..20], &5.0f32.to_le_bytes());
        assert_eq!(&buf[20..24], &6.0f32.to_le_bytes());
        assert_eq!(&buf[24..28], &7.0f32.to_le_bytes());
        assert_eq!(&buf[28..32], &8.0f32.to_le_bytes());
        // widths = vec4 (2 real + 2 padding zeros)
        assert_eq!(&buf[32..36], &0.5f32.to_le_bytes());
        assert_eq!(&buf[36..40], &0.75f32.to_le_bytes());
        assert_eq!(&buf[40..44], &[0u8; 4]);
        assert_eq!(&buf[44..48], &[0u8; 4]);
    }

    #[test]
    fn std140_glyph_ubo_meta_encodes_advance_and_stroke_count() {
        // v0.5.2 (S6.3.2b) 新 layout: 末尾 vec4 uMeta = (advance, stroke_count_f32, 0, 0)。
        let params = MetaFontParams::sans_regular();
        let payload = pack_glyph_for_shader('森', &params).unwrap();
        let bytes = payload.to_std140_bytes();
        assert_eq!(bytes.len(), GLYPH_UBO_SIZE);
        // uMeta の offset: 16 stroke × 48B + 16B pen = 784
        let meta_offset = MAX_STROKES * AnalyticalStroke::STD140_SIZE + AnalyticalPen::STD140_SIZE;
        assert_eq!(meta_offset, 784);
        // advance
        assert_eq!(
            &bytes[meta_offset..meta_offset + 4],
            &payload.advance.to_le_bytes()
        );
        // stroke_count as f32
        #[allow(clippy::cast_precision_loss)]
        let expected_count = payload.strokes.len() as f32;
        assert_eq!(
            &bytes[meta_offset + 4..meta_offset + 8],
            &expected_count.to_le_bytes()
        );
        // 残り 8B pad = 0
        assert_eq!(&bytes[meta_offset + 8..meta_offset + 16], &[0u8; 8]);
    }

    #[test]
    fn std140_glyph_ubo_pads_unused_stroke_slots_to_zero() {
        // 森 stroke_count = 12、残り 4 slot は zero-fill されるべき。
        let params = MetaFontParams::sans_regular();
        let payload = pack_glyph_for_shader('森', &params).unwrap();
        let bytes = payload.to_std140_bytes();
        let actual_stroke_bytes = payload.strokes.len() * AnalyticalStroke::STD140_SIZE;
        let stroke_area_end = MAX_STROKES * AnalyticalStroke::STD140_SIZE;
        // 未使用 slot 領域は zero fill 期待。
        for (offset, byte) in bytes
            .iter()
            .enumerate()
            .take(stroke_area_end)
            .skip(actual_stroke_bytes)
        {
            assert_eq!(*byte, 0, "stroke slot padding at offset {offset} must be 0");
        }
    }

    // ------------------------------------------------------------------------
    // S6.3.2b.3 — MultiGlyphPayload UBO chunks (instanced rendering)
    // ------------------------------------------------------------------------

    #[test]
    fn multi_glyph_ubo_size_is_fixed_12816_bytes() {
        assert_eq!(MAX_GLYPHS_PER_DRAW, 16);
        assert_eq!(MULTI_GLYPH_UBO_SIZE, 16 * 800 + 16);
        assert_eq!(MULTI_GLYPH_UBO_SIZE, 12_816);
    }

    #[test]
    fn multi_glyph_ubo_chunks_fits_in_16kb_webgl2_min_spec() {
        // WebGL2 GL_MAX_UNIFORM_BLOCK_SIZE 最低保証は 16 KB (16,384 byte)。
        // clippy::assertions_on_constants を const block で満たす。
        const _: () = assert!(
            MULTI_GLYPH_UBO_SIZE <= 16 * 1024,
            "MULTI_GLYPH_UBO_SIZE > 16 KB webgl2 min"
        );
    }

    #[test]
    fn multi_glyph_short_text_produces_single_chunk() {
        let params = MetaFontParams::sans_regular();
        // 3 glyph 森明林 → 1 chunk (16 以下)
        let payload = pack_string_for_shader("森明林", &params).unwrap();
        assert_eq!(payload.chunk_count(), 1);
        let chunks = payload.to_std140_ubo_chunks();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), MULTI_GLYPH_UBO_SIZE);
    }

    #[test]
    fn multi_glyph_16_glyphs_still_single_chunk() {
        let params = MetaFontParams::sans_regular();
        // ちょうど 16 glyph = 1 chunk 満載。
        let text = "一二三四五六七八九十百千万木火水";
        assert_eq!(text.chars().count(), 16);
        let payload = pack_string_for_shader(text, &params).unwrap();
        assert_eq!(payload.glyph_count(), 16);
        assert_eq!(payload.chunk_count(), 1);
        assert_eq!(payload.to_std140_ubo_chunks().len(), 1);
    }

    #[test]
    fn multi_glyph_17_glyphs_splits_to_two_chunks() {
        let params = MetaFontParams::sans_regular();
        // 17 glyph = 16 + 1 → 2 chunks。
        let text = "一二三四五六七八九十百千万木火水土";
        assert_eq!(text.chars().count(), 17);
        let payload = pack_string_for_shader(text, &params).unwrap();
        assert_eq!(payload.glyph_count(), 17);
        assert_eq!(payload.chunk_count(), 2);
        let chunks = payload.to_std140_ubo_chunks();
        assert_eq!(chunks.len(), 2);
        for chunk in &chunks {
            assert_eq!(chunk.len(), MULTI_GLYPH_UBO_SIZE);
        }
    }

    #[test]
    fn multi_glyph_flat_meta_encodes_advance_stroke_count_xoffset() {
        // v0.5.4 flat layout: uMetas[i] at offset MULTI_GLYPH_STROKE_AREA_SIZE + i * 32
        // GlyphMeta = { vec4 pen (16B); vec4 meta (16B) } = 32B
        // meta = (advance, stroke_count_f, x_offset, 0)
        let params = MetaFontParams::sans_regular();
        let payload = pack_string_for_shader("森明林", &params).unwrap();
        let chunk = &payload.to_std140_ubo_chunks()[0];
        for (i, expected_layout) in payload.layout.iter().enumerate().take(3) {
            // pen (16B) の後に meta が来る
            let meta_offset = MULTI_GLYPH_STROKE_AREA_SIZE + i * 32 + 16;
            assert_eq!(
                &chunk[meta_offset..meta_offset + 4],
                &expected_layout.advance.to_le_bytes(),
                "slot {i}: meta.x (advance) mismatch"
            );
            #[allow(clippy::cast_precision_loss)]
            let sc_f = payload.glyphs[i].strokes.len() as f32;
            assert_eq!(
                &chunk[meta_offset + 4..meta_offset + 8],
                &sc_f.to_le_bytes(),
                "slot {i}: meta.y (stroke_count) mismatch"
            );
            assert_eq!(
                &chunk[meta_offset + 8..meta_offset + 12],
                &expected_layout.x_offset.to_le_bytes(),
                "slot {i}: meta.z (x_offset) mismatch"
            );
            assert_eq!(
                &chunk[meta_offset + 12..meta_offset + 16],
                &[0u8; 4],
                "slot {i}: meta.w padding must be 0"
            );
        }
    }

    #[test]
    fn multi_glyph_utext_meta_encodes_total_advance_and_chunk_count() {
        let params = MetaFontParams::sans_regular();
        let payload = pack_string_for_shader("森明林", &params).unwrap();
        let chunk = &payload.to_std140_ubo_chunks()[0];
        // v0.5.4 flat layout: uTextMeta at MULTI_GLYPH_STROKE_AREA_SIZE + MULTI_GLYPH_METAS_AREA_SIZE
        let text_meta_offset = MULTI_GLYPH_STROKE_AREA_SIZE + MULTI_GLYPH_METAS_AREA_SIZE;
        assert_eq!(text_meta_offset, 12_800);
        assert_eq!(
            &chunk[text_meta_offset..text_meta_offset + 4],
            &payload.total_advance.to_le_bytes()
        );
        #[allow(clippy::cast_precision_loss)]
        let count_f = 3.0f32;
        assert_eq!(
            &chunk[text_meta_offset + 4..text_meta_offset + 8],
            &count_f.to_le_bytes()
        );
        assert_eq!(
            &chunk[text_meta_offset + 8..text_meta_offset + 16],
            &[0u8; 8]
        );
    }

    #[test]
    fn multi_glyph_ubo_second_chunk_encodes_partial_glyph_count() {
        let params = MetaFontParams::sans_regular();
        let text = "一二三四五六七八九十百千万木火水土";
        let payload = pack_string_for_shader(text, &params).unwrap();
        let chunks = payload.to_std140_ubo_chunks();
        assert_eq!(chunks.len(), 2);
        let text_meta_offset = MULTI_GLYPH_STROKE_AREA_SIZE + MULTI_GLYPH_METAS_AREA_SIZE;
        #[allow(clippy::cast_precision_loss)]
        let one_f = 1.0f32;
        assert_eq!(
            &chunks[1][text_meta_offset + 4..text_meta_offset + 8],
            &one_f.to_le_bytes(),
            "chunk 1 must have glyph_count=1 in uTextMeta.y"
        );
    }

    #[test]
    fn multi_glyph_flat_first_stroke_of_first_glyph_is_bit_exact() {
        // v0.5.4 flat: uStrokes[0] = 森 stroke[0] control points at offset 0..48
        let params = MetaFontParams::sans_regular();
        let payload = pack_string_for_shader("森", &params).unwrap();
        let chunk = &payload.to_std140_ubo_chunks()[0];
        let stroke0 = &payload.glyphs[0].strokes[0];
        // p01 = vec4(p0.x, p0.y, p1.x, p1.y) at offset 0..16
        assert_eq!(&chunk[0..4], &stroke0.p0[0].to_le_bytes());
        assert_eq!(&chunk[4..8], &stroke0.p0[1].to_le_bytes());
        assert_eq!(&chunk[8..12], &stroke0.p1[0].to_le_bytes());
        assert_eq!(&chunk[12..16], &stroke0.p1[1].to_le_bytes());
        // widths at offset 32..48
        assert_eq!(&chunk[32..36], &stroke0.half_width_start.to_le_bytes());
        assert_eq!(&chunk[36..40], &stroke0.half_width_end.to_le_bytes());
    }

    #[test]
    fn multi_glyph_flat_second_glyph_strokes_at_glyph_offset_16() {
        // v0.5.4: glyph 1 の strokes は uStrokes[16..32]、offset = 16 × 48 = 768
        let params = MetaFontParams::sans_regular();
        let payload = pack_string_for_shader("森明", &params).unwrap();
        let chunk = &payload.to_std140_ubo_chunks()[0];
        let glyph1_stroke0 = &payload.glyphs[1].strokes[0];
        let glyph1_stroke0_offset = 16 * AnalyticalStroke::STD140_SIZE; // 768
        assert_eq!(
            &chunk[glyph1_stroke0_offset..glyph1_stroke0_offset + 4],
            &glyph1_stroke0.p0[0].to_le_bytes(),
            "glyph 1 stroke 0 must start at flat offset 768"
        );
    }
}

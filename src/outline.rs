//! Outline (polygon contour) rendering path — L0 shader API (v0.6.0 S6.3.3)。
//!
//! S6.3.3 milestone: TTF-derived polygon outline を fragment shader が
//! winding number test で per-pixel evaluation する path。既存 stroke+pen
//! envelope path (S6.3.1-b.4) と並列に提供、real font-derived character
//! shapes を law-only shipping (polygon points = 法則) で render 可能に。
//!
//! # L0/L1 分離 design (2026-07-30 制定)
//!
//! - **L0 (このcrate)** = `OutlineGlyph` 型 + `pack_outline_for_shader` +
//!   GLSL shader (`shaders/analytical_outline.{vert,frag}.glsl`)。
//!   data 非依存の shader path infrastructure。
//! - **L1 (outline provider)** = BIZ UDPGothic 等 TTF-derived outline data
//!   の provider (`regular_outline(ch) -> OutlineGlyph` の形)。
//! - **Consumer** = L0 + L1 両方 import、L1 で data 取得 → L0 で shader
//!   pack + render。ALICE-Font 単独では synthetic sample のみ使用可。
//!
//! # runtime texture upload = 0 byte
//!
//! Polygon points (contour list) は uniform buffer 経由の数値、bitmap / atlas
//! / TTF outline binary 一切なし。「データを送るな、法則を送れ」原則遵守。
//!
//! # 使い方
//!
//! ```
//! use alice_font::outline::{OutlineGlyph, pack_outline_for_shader};
//!
//! // Synthetic 三 (3 rectangles) — 実 BIZ UDPGothic data は L1 (outline provider) 経由。
//! let three = OutlineGlyph {
//!     codepoint: '三',
//!     contours: vec![
//!         vec![[0.15, 0.75], [0.85, 0.75], [0.85, 0.85], [0.15, 0.85]],
//!         vec![[0.20, 0.45], [0.80, 0.45], [0.80, 0.55], [0.20, 0.55]],
//!         vec![[0.10, 0.15], [0.90, 0.15], [0.90, 0.25], [0.10, 0.25]],
//!     ],
//!     advance: 1.0,
//! };
//! let payload = pack_outline_for_shader(&three).unwrap();
//! assert_eq!(payload.total_points(), 12);
//! assert_eq!(payload.contour_count(), 3);
//! ```
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec::Vec;

/// UBO 内 polygon point 最大数 (packed 2/vec4 = 128 vec4 slot、std140 で 2048B)。
pub const OUTLINE_MAX_POINTS: usize = 256;

/// UBO 内 contour 最大数 (各 contour = start/count/0/0 vec4 = 16 slot、std140 256B)。
///
/// v0.6.7: 12 → 16 に拡張 (護 = 13 contour が truncate されていた事案解消)。
/// BIZ UDPGothic 実測で 護 (13) が唯一の 12 超過、16 で完全カバー。
pub const OUTLINE_MAX_CONTOURS: usize = 16;

/// GLSL `layout(std140) uniform OutlineBlock` に対応する fixed-size UBO 総 byte 数。
///
/// - point 領域 (128 vec4 × 16 = 2048B、packed 2 point/vec4)
/// - contour info (16 vec4 × 16 = 256B、std140 alignment に応じて padding)
/// - outline meta vec4 (16B)
///
/// 合計 **2,320 byte** fixed (v0.6.7)。WebGL2 GL_MAX_UNIFORM_BLOCK_SIZE min 16KB 内。
pub const OUTLINE_UBO_SIZE: usize = OUTLINE_POINTS_AREA_SIZE + OUTLINE_CONTOURS_AREA_SIZE + 16;

/// UBO 内 packed point 領域 (2 point/vec4 × 128 vec4 = 2,048B)。
pub const OUTLINE_POINTS_AREA_SIZE: usize = (OUTLINE_MAX_POINTS / 2) * 16;

/// UBO 内 contour info 領域 (v0.6.7: 16 vec4 × 16 = **256B**、was 192B)。
pub const OUTLINE_CONTOURS_AREA_SIZE: usize = OUTLINE_MAX_CONTOURS * 16;

/// TTF-derived polygon outline (data 非依存、法則 shipping)。
///
/// `contours` = 閉じた polygon list。各 contour は最初と最後の点が implicit
/// に接続される (winding number 計算時に close edge を追加)。
/// `contours[i]` が外形、以降は holes (e.g. `田` = 外 + 4 内)。
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutlineGlyph {
    /// Unicode codepoint。
    pub codepoint: char,
    /// 各 contour = 閉じた polygon (最初と最後の点は implicit に接続)。
    pub contours: Vec<Vec<[f32; 2]>>,
    /// Advance width (em unit)。
    pub advance: f32,
}

impl OutlineGlyph {
    /// 全 contour の point 総数。
    #[must_use]
    pub fn total_points(&self) -> usize {
        self.contours.iter().map(Vec::len).sum()
    }

    /// Contour 数。
    #[must_use]
    pub fn contour_count(&self) -> usize {
        self.contours.len()
    }

    /// Bounding box `(xmin, ymin, xmax, ymax)`。全 contour 全 point を走査。
    /// 空 outline (contour または point ゼロ) は `(0, 0, 0, 0)` を返す。
    #[must_use]
    pub fn bounding_box(&self) -> [f32; 4] {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for contour in &self.contours {
            for &[x, y] in contour {
                if x < min_x {
                    min_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if x > max_x {
                    max_x = x;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
        if min_x == f32::MAX {
            [0.0, 0.0, 0.0, 0.0]
        } else {
            [min_x, min_y, max_x, max_y]
        }
    }
}

/// Fragment shader ready outline payload。
///
/// `pack_outline_for_shader` が生成、`to_std140_bytes` で UBO 用 byte 列に変換。
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutlineGlyphPayload {
    /// 元 OutlineGlyph の参照 (clone、to_std140_bytes で消費)。
    pub glyph: OutlineGlyph,
    /// Bounding box (JS 側 debug + zoom fit 用)。
    pub bounding_box: [f32; 4],
}

impl OutlineGlyphPayload {
    #[must_use]
    pub fn total_points(&self) -> usize {
        self.glyph.total_points()
    }

    #[must_use]
    pub fn contour_count(&self) -> usize {
        self.glyph.contour_count()
    }

    /// UBO byte size ([`OUTLINE_UBO_SIZE`] = 2,256 fixed)。
    #[must_use]
    pub fn payload_bytes(&self) -> usize {
        OUTLINE_UBO_SIZE
    }

    /// std140 UBO 用の byte 列を出力 (little-endian、2,256B fixed)。
    ///
    /// Layout (`shaders/analytical_outline.frag.glsl` と bit-exact 一致):
    /// - Offset 0..2,048: `vec4 uPoints[128]` (256 point packed 2 per vec4)
    /// - Offset 2,048..2,240: `vec4 uContourInfo[12]` (start, count, 0, 0 as float × 12)
    /// - Offset 2,240..2,256: `vec4 uOutlineMeta` (contour_count_f, total_points_f, advance, 0)
    #[must_use]
    pub fn to_std140_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(OUTLINE_UBO_SIZE);

        // Section 1: packed points (128 vec4 = 2048B、2 point/vec4)
        // Flatten contours → point list.
        let mut flat_points: Vec<[f32; 2]> = Vec::new();
        for contour in &self.glyph.contours {
            for &pt in contour {
                if flat_points.len() < OUTLINE_MAX_POINTS {
                    flat_points.push(pt);
                }
            }
        }
        // Pad to MAX_POINTS with zeros (未使用 slot は 0)。
        while flat_points.len() < OUTLINE_MAX_POINTS {
            flat_points.push([0.0, 0.0]);
        }
        // 2 point per vec4 packed。
        for chunk in flat_points.chunks(2) {
            out.extend_from_slice(&chunk[0][0].to_le_bytes());
            out.extend_from_slice(&chunk[0][1].to_le_bytes());
            out.extend_from_slice(&chunk[1][0].to_le_bytes());
            out.extend_from_slice(&chunk[1][1].to_le_bytes());
        }
        debug_assert_eq!(out.len(), OUTLINE_POINTS_AREA_SIZE);

        // Section 2: contour info (12 vec4 = 192B、per contour: start, count, 0, 0 as f32)
        let mut cursor = 0u32;
        for contour in &self.glyph.contours {
            if out.len() >= OUTLINE_POINTS_AREA_SIZE + OUTLINE_CONTOURS_AREA_SIZE {
                break; // MAX_CONTOURS 到達
            }
            #[allow(clippy::cast_precision_loss)]
            let start_f = cursor as f32;
            #[allow(clippy::cast_precision_loss)]
            let count_f = contour.len().min(OUTLINE_MAX_POINTS - cursor as usize) as f32;
            out.extend_from_slice(&start_f.to_le_bytes());
            out.extend_from_slice(&count_f.to_le_bytes());
            out.extend_from_slice(&0f32.to_le_bytes());
            out.extend_from_slice(&0f32.to_le_bytes());
            #[allow(clippy::cast_possible_truncation)]
            let contour_len_u32 = contour.len() as u32;
            cursor = (cursor + contour_len_u32).min(OUTLINE_MAX_POINTS as u32);
        }
        // Pad remaining contour slots with zero (16B each)。
        let remaining_slots = OUTLINE_MAX_CONTOURS
            .saturating_sub(self.glyph.contour_count().min(OUTLINE_MAX_CONTOURS));
        for _ in 0..remaining_slots {
            out.extend_from_slice(&[0u8; 16]);
        }
        debug_assert_eq!(
            out.len(),
            OUTLINE_POINTS_AREA_SIZE + OUTLINE_CONTOURS_AREA_SIZE
        );

        // Section 3: uOutlineMeta vec4 (16B)
        // (contour_count_f, total_points_f, advance, 0)
        #[allow(clippy::cast_precision_loss)]
        let contour_count_f = self.contour_count().min(OUTLINE_MAX_CONTOURS) as f32;
        #[allow(clippy::cast_precision_loss)]
        let total_points_f = self.total_points().min(OUTLINE_MAX_POINTS) as f32;
        out.extend_from_slice(&contour_count_f.to_le_bytes());
        out.extend_from_slice(&total_points_f.to_le_bytes());
        out.extend_from_slice(&self.glyph.advance.to_le_bytes());
        out.extend_from_slice(&0f32.to_le_bytes());

        debug_assert_eq!(out.len(), OUTLINE_UBO_SIZE);
        out
    }
}

/// Outline を fragment-shader-ready payload に pack。
///
/// # Returns
/// - `Some(payload)` — contour count >= 1 かつ total points >= 3 (三角形以上)
/// - `None` — 空 outline or point 数不足
#[must_use]
pub fn pack_outline_for_shader(outline: &OutlineGlyph) -> Option<OutlineGlyphPayload> {
    if outline.contours.is_empty() {
        return None;
    }
    let total = outline.total_points();
    if total < 3 {
        return None;
    }
    Some(OutlineGlyphPayload {
        glyph: outline.clone(),
        bounding_box: outline.bounding_box(),
    })
}

// ============================================================================
// S6.3.3 (v0.6.1) — Multi-outline instanced rendering (chunk 分割)
// ============================================================================

/// 1 chunk に pack できる最大 outline glyph 数 (WebGL2 16KB UBO min 内)。
///
/// v0.6.3: 6 → 4 に削減 (per-glyph point 数を 1.5 倍 256 → 384 にしたため、
/// UBO fit 維持)。BIZ UDPGothic 実測で 議/感/務/義/海 等 300-400 pt の
/// 複雑 kanji が 384 cap 内に収まり truncation artifact 解消。
pub const MULTI_OUTLINE_MAX_GLYPHS: usize = 4;

/// Multi-outline UBO の per-glyph 最大 point 数。
///
/// v0.6.3: 256 → 384 に拡張 (BIZ UDPGothic complex kanji 実測結果反映)。
/// 感 (371) / 議 (366) / 務 (355) / 義 (320) / 海 (299) が 384 cap 内に
/// 収まり、全 measured 常用 kanji が truncate なしで完全描画可能。
pub const MULTI_OUTLINE_MAX_POINTS_PER_GLYPH: usize = 384;

/// Multi-outline UBO の per-glyph 最大 contour 数。
///
/// v0.6.7: 12 → 16 に拡張 (護 = 13 contour が truncate されていた事案解消)。
/// BIZ UDPGothic JIS L1 3,344 char 実測で 護 (13) が唯一の 12 超過、16 で
/// 完全カバー。UBO CONTOURS_AREA は +256B (768→1,024B)、total UBO は +256B。
pub const MULTI_OUTLINE_MAX_CONTOURS_PER_GLYPH: usize = 16;

/// Multi-outline UBO 内 points area 総 byte 数。
///
/// v0.6.3: 4 glyph × 384 point × 8B (packed 2/vec4) = **12,288B**
/// (=6 × 256 と同一、glyph/point tradeoff で 総量は偶然一致)。
pub const MULTI_OUTLINE_POINTS_AREA_SIZE: usize =
    (MULTI_OUTLINE_MAX_GLYPHS * MULTI_OUTLINE_MAX_POINTS_PER_GLYPH / 2) * 16;

/// Multi-outline UBO 内 contour infos area (flat 4 × 16 vec4 = 64 vec4 = **1,024B**、v0.6.7)。
pub const MULTI_OUTLINE_CONTOURS_AREA_SIZE: usize =
    MULTI_OUTLINE_MAX_GLYPHS * MULTI_OUTLINE_MAX_CONTOURS_PER_GLYPH * 16;

/// Multi-outline UBO 内 glyph metas area (4 vec4 = **64B**)。
pub const MULTI_OUTLINE_METAS_AREA_SIZE: usize = MULTI_OUTLINE_MAX_GLYPHS * 16;

/// Multi-outline UBO 総 byte 数 (fixed per chunk = **13,392B**、v0.6.7)。
///
/// v0.6.7 layout (`shaders/analytical_multi_outline.frag.glsl` と bit-exact 一致):
/// - Offset 0..12,288: `vec4 uPoints[768]` (flat 4 glyph × 384 point packed 2/vec4)
/// - Offset 12,288..13,312: `vec4 uContourInfos[64]` (flat 4 × 16 contour info)
/// - Offset 13,312..13,376: `vec4 uGlyphMetas[4]` (advance, contour_count, x_offset, 0)
/// - Offset 13,376..13,392: `vec4 uTextMeta` (total_advance, chunk_glyph_count, 0, 0)
///
/// 13,392 / 16,384 (WebGL2 min UBO) = 81.7% 使用、safe headroom 2,992B。
pub const MULTI_OUTLINE_UBO_SIZE: usize = MULTI_OUTLINE_POINTS_AREA_SIZE
    + MULTI_OUTLINE_CONTOURS_AREA_SIZE
    + MULTI_OUTLINE_METAS_AREA_SIZE
    + 16;

/// 単一 glyph の text layout 内配置情報 (em 座標)。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct OutlineGlyphLayout {
    /// この glyph の左端 x 座標 (em 単位、text 全体 0 起点)。
    pub x_offset: f32,
    /// この glyph の advance (em 単位)。
    pub advance: f32,
}

/// Multi-glyph outline text payload (v0.6.1 S6.3.3)。
///
/// Instanced rendering + chunk 分割で任意長 text 対応。1 chunk = 最大
/// [`MULTI_OUTLINE_MAX_GLYPHS`] (4) glyph、超過分は次 chunk へ。
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MultiOutlineGlyphPayload {
    /// 各 glyph の outline (reading order、未登録 char は skip)。
    pub glyphs: Vec<OutlineGlyph>,
    /// 各 glyph の text layout 配置。
    pub layout: Vec<OutlineGlyphLayout>,
    /// Text 全体の advance (em 単位)。
    pub total_advance: f32,
}

impl MultiOutlineGlyphPayload {
    /// Glyph 数。
    #[must_use]
    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }

    /// Chunk 数 (`ceil(glyph_count / MULTI_OUTLINE_MAX_GLYPHS)`)。
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.glyphs.len().div_ceil(MULTI_OUTLINE_MAX_GLYPHS)
    }

    /// std140 UBO chunks 生成 (each = 9,872 byte fixed)。
    ///
    /// Flat layout (`shaders/analytical_multi_outline.frag.glsl` と一致):
    /// - Section 1: 全 glyph の全 point (flat)
    /// - Section 2: 全 glyph の全 contour info (flat)
    /// - Section 3: per-glyph meta (advance, contour_count, x_offset, 0)
    /// - Section 4: uTextMeta (total_advance, chunk_glyph_count, 0, 0)
    #[must_use]
    pub fn to_std140_ubo_chunks(&self) -> Vec<Vec<u8>> {
        let n_chunks = self.chunk_count();
        let mut chunks = Vec::with_capacity(n_chunks);
        for chunk_idx in 0..n_chunks {
            let start = chunk_idx * MULTI_OUTLINE_MAX_GLYPHS;
            let end = (start + MULTI_OUTLINE_MAX_GLYPHS).min(self.glyphs.len());
            let chunk_glyph_count = end - start;
            let mut out = Vec::with_capacity(MULTI_OUTLINE_UBO_SIZE);

            // ---------- Section 1: points (8 glyph × 128 point、flat packed 2/vec4) ----------
            for i in start..end {
                write_glyph_points_padded(&mut out, &self.glyphs[i]);
            }
            // 未使用 glyph slot は zero-fill (128 point × 8B = 1024B / glyph)
            for _ in chunk_glyph_count..MULTI_OUTLINE_MAX_GLYPHS {
                out.extend_from_slice(&[0u8; MULTI_OUTLINE_MAX_POINTS_PER_GLYPH * 8]);
            }
            debug_assert_eq!(out.len(), MULTI_OUTLINE_POINTS_AREA_SIZE);

            // ---------- Section 2: contour infos (8 × 12 vec4、flat) ----------
            for i in start..end {
                write_glyph_contour_infos_padded(&mut out, &self.glyphs[i]);
            }
            for _ in chunk_glyph_count..MULTI_OUTLINE_MAX_GLYPHS {
                out.extend_from_slice(&[0u8; MULTI_OUTLINE_MAX_CONTOURS_PER_GLYPH * 16]);
            }
            debug_assert_eq!(
                out.len(),
                MULTI_OUTLINE_POINTS_AREA_SIZE + MULTI_OUTLINE_CONTOURS_AREA_SIZE
            );

            // ---------- Section 3: per-glyph meta (advance, contour_count, x_offset, 0) ----------
            for i in start..end {
                let glyph = &self.glyphs[i];
                let layout = self.layout[i];
                out.extend_from_slice(&layout.advance.to_le_bytes());
                #[allow(clippy::cast_precision_loss)]
                let contour_count_f = glyph
                    .contour_count()
                    .min(MULTI_OUTLINE_MAX_CONTOURS_PER_GLYPH)
                    as f32;
                out.extend_from_slice(&contour_count_f.to_le_bytes());
                out.extend_from_slice(&layout.x_offset.to_le_bytes());
                out.extend_from_slice(&0f32.to_le_bytes());
            }
            for _ in chunk_glyph_count..MULTI_OUTLINE_MAX_GLYPHS {
                out.extend_from_slice(&[0u8; 16]);
            }
            debug_assert_eq!(
                out.len(),
                MULTI_OUTLINE_POINTS_AREA_SIZE
                    + MULTI_OUTLINE_CONTOURS_AREA_SIZE
                    + MULTI_OUTLINE_METAS_AREA_SIZE
            );

            // ---------- Section 4: uTextMeta ----------
            out.extend_from_slice(&self.total_advance.to_le_bytes());
            #[allow(clippy::cast_precision_loss)]
            let count_f = chunk_glyph_count as f32;
            out.extend_from_slice(&count_f.to_le_bytes());
            out.extend_from_slice(&0f32.to_le_bytes());
            out.extend_from_slice(&0f32.to_le_bytes());

            debug_assert_eq!(out.len(), MULTI_OUTLINE_UBO_SIZE);
            chunks.push(out);
        }
        chunks
    }
}

/// Write per-glyph point section (padded to `MULTI_OUTLINE_MAX_POINTS_PER_GLYPH`、
/// 2 point/vec4 packed = 128 point × 8B = 1,024B)。
fn write_glyph_points_padded(out: &mut Vec<u8>, glyph: &OutlineGlyph) {
    let mut flat_points: Vec<[f32; 2]> = Vec::new();
    for contour in &glyph.contours {
        for &pt in contour {
            if flat_points.len() < MULTI_OUTLINE_MAX_POINTS_PER_GLYPH {
                flat_points.push(pt);
            }
        }
    }
    while flat_points.len() < MULTI_OUTLINE_MAX_POINTS_PER_GLYPH {
        flat_points.push([0.0, 0.0]);
    }
    for chunk in flat_points.chunks(2) {
        out.extend_from_slice(&chunk[0][0].to_le_bytes());
        out.extend_from_slice(&chunk[0][1].to_le_bytes());
        out.extend_from_slice(&chunk[1][0].to_le_bytes());
        out.extend_from_slice(&chunk[1][1].to_le_bytes());
    }
}

/// Write per-glyph contour info section (padded to `MULTI_OUTLINE_MAX_CONTOURS_PER_GLYPH`
/// = 12 vec4 = 192B、per contour: start_f, count_f, 0, 0)。
fn write_glyph_contour_infos_padded(out: &mut Vec<u8>, glyph: &OutlineGlyph) {
    let mut cursor = 0u32;
    let mut written = 0;
    for contour in &glyph.contours {
        if written >= MULTI_OUTLINE_MAX_CONTOURS_PER_GLYPH {
            break;
        }
        #[allow(clippy::cast_precision_loss)]
        let start_f = cursor as f32;
        let remaining = (MULTI_OUTLINE_MAX_POINTS_PER_GLYPH as u32).saturating_sub(cursor) as usize;
        #[allow(clippy::cast_precision_loss)]
        let count_f = contour.len().min(remaining) as f32;
        out.extend_from_slice(&start_f.to_le_bytes());
        out.extend_from_slice(&count_f.to_le_bytes());
        out.extend_from_slice(&0f32.to_le_bytes());
        out.extend_from_slice(&0f32.to_le_bytes());
        #[allow(clippy::cast_possible_truncation)]
        let contour_u32 = contour.len() as u32;
        cursor = (cursor + contour_u32).min(MULTI_OUTLINE_MAX_POINTS_PER_GLYPH as u32);
        written += 1;
    }
    for _ in written..MULTI_OUTLINE_MAX_CONTOURS_PER_GLYPH {
        out.extend_from_slice(&[0u8; 16]);
    }
}

/// Text (複数文字) を outline `MultiOutlineGlyphPayload` に pack。
///
/// `provider` は char ごとに outline を返す callback (synthetic sample や
/// outline provider 経由の BIZ UDPGothic data 等、L0 は data 非依存)。
///
/// # Returns
/// - `Some(payload)` — 1 char 以上 pack 成功
/// - `None` — text 空 or 全 char が provider から `None`
#[must_use]
pub fn pack_outline_string(
    text: &str,
    provider: impl Fn(char) -> Option<OutlineGlyph>,
) -> Option<MultiOutlineGlyphPayload> {
    let mut glyphs = Vec::new();
    let mut layout = Vec::new();
    let mut cursor = 0.0f32;
    for ch in text.chars() {
        if let Some(outline) = provider(ch) {
            let advance = outline.advance;
            layout.push(OutlineGlyphLayout {
                x_offset: cursor,
                advance,
            });
            cursor += advance;
            glyphs.push(outline);
        }
    }
    if glyphs.is_empty() {
        None
    } else {
        Some(MultiOutlineGlyphPayload {
            glyphs,
            layout,
            total_advance: cursor,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec;

    fn sample_three() -> OutlineGlyph {
        // 三 = 3 horizontal rectangles (synthetic sample、not BIZ UDPGothic)。
        OutlineGlyph {
            codepoint: '三',
            contours: vec![
                vec![[0.15, 0.75], [0.85, 0.75], [0.85, 0.85], [0.15, 0.85]],
                vec![[0.20, 0.45], [0.80, 0.45], [0.80, 0.55], [0.20, 0.55]],
                vec![[0.10, 0.15], [0.90, 0.15], [0.90, 0.25], [0.10, 0.25]],
            ],
            advance: 1.0,
        }
    }

    #[test]
    fn ubo_size_constant_is_2320_bytes() {
        // v0.6.7: MAX_CONTOURS 12→16 で CONTOURS_AREA 192→256 → total 2,256→2,320
        assert_eq!(OUTLINE_UBO_SIZE, 2_320);
        assert_eq!(OUTLINE_POINTS_AREA_SIZE, 2_048);
        assert_eq!(OUTLINE_CONTOURS_AREA_SIZE, 256);
    }

    #[test]
    fn ubo_fits_in_16kb_webgl2_min_spec() {
        const _: () = assert!(
            OUTLINE_UBO_SIZE <= 16 * 1024,
            "OUTLINE_UBO_SIZE > 16 KB WebGL2 min"
        );
    }

    #[test]
    fn outline_glyph_total_points_and_contour_count() {
        let three = sample_three();
        assert_eq!(three.total_points(), 12);
        assert_eq!(three.contour_count(), 3);
    }

    #[test]
    fn outline_glyph_bounding_box() {
        let three = sample_three();
        let bb = three.bounding_box();
        assert_eq!(bb, [0.10, 0.15, 0.90, 0.85]);
    }

    #[test]
    fn pack_outline_synthetic_three_succeeds() {
        let three = sample_three();
        let payload = pack_outline_for_shader(&three).unwrap();
        assert_eq!(payload.total_points(), 12);
        assert_eq!(payload.contour_count(), 3);
        assert_eq!(payload.payload_bytes(), OUTLINE_UBO_SIZE);
    }

    #[test]
    fn pack_outline_empty_returns_none() {
        let empty = OutlineGlyph {
            codepoint: '?',
            contours: vec![],
            advance: 1.0,
        };
        assert!(pack_outline_for_shader(&empty).is_none());
    }

    #[test]
    fn pack_outline_too_few_points_returns_none() {
        let tiny = OutlineGlyph {
            codepoint: '?',
            contours: vec![vec![[0.0, 0.0], [1.0, 0.0]]], // 2 points < 3
            advance: 1.0,
        };
        assert!(pack_outline_for_shader(&tiny).is_none());
    }

    #[test]
    fn std140_bytes_size_is_fixed_2320() {
        // v0.6.7: MAX_CONTOURS 12→16 で total 2,256 → 2,320
        let payload = pack_outline_for_shader(&sample_three()).unwrap();
        let bytes = payload.to_std140_bytes();
        assert_eq!(bytes.len(), OUTLINE_UBO_SIZE);
        assert_eq!(bytes.len(), 2_320);
    }

    #[test]
    fn std140_first_point_bit_exact() {
        // 三 の最初の point (contour 0 point 0) = (0.15, 0.75)、offset 0..8。
        let payload = pack_outline_for_shader(&sample_three()).unwrap();
        let bytes = payload.to_std140_bytes();
        assert_eq!(&bytes[0..4], &0.15f32.to_le_bytes());
        assert_eq!(&bytes[4..8], &0.75f32.to_le_bytes());
        // 2 point per vec4: 次 point (contour 0 point 1) = (0.85, 0.75) at offset 8..16
        assert_eq!(&bytes[8..12], &0.85f32.to_le_bytes());
        assert_eq!(&bytes[12..16], &0.75f32.to_le_bytes());
    }

    #[test]
    fn std140_contour_info_encodes_start_and_count() {
        // 三: contour 0 = 4 pt (start=0), contour 1 = 4 pt (start=4), contour 2 = 4 pt (start=8)。
        let payload = pack_outline_for_shader(&sample_three()).unwrap();
        let bytes = payload.to_std140_bytes();
        // Contour info area: offset 2048..
        // Contour 0: (0, 4, 0, 0)
        assert_eq!(&bytes[2048..2052], &0.0f32.to_le_bytes());
        assert_eq!(&bytes[2052..2056], &4.0f32.to_le_bytes());
        // Contour 1: (4, 4, 0, 0)
        assert_eq!(&bytes[2064..2068], &4.0f32.to_le_bytes());
        assert_eq!(&bytes[2068..2072], &4.0f32.to_le_bytes());
        // Contour 2: (8, 4, 0, 0)
        assert_eq!(&bytes[2080..2084], &8.0f32.to_le_bytes());
        assert_eq!(&bytes[2084..2088], &4.0f32.to_le_bytes());
    }

    #[test]
    fn std140_meta_encodes_contour_count_total_points_advance() {
        // v0.6.7: uOutlineMeta at offset 2304..2320 (was 2240..2256)
        let payload = pack_outline_for_shader(&sample_three()).unwrap();
        let bytes = payload.to_std140_bytes();
        assert_eq!(&bytes[2304..2308], &3.0f32.to_le_bytes()); // contour_count
        assert_eq!(&bytes[2308..2312], &12.0f32.to_le_bytes()); // total_points
        assert_eq!(&bytes[2312..2316], &1.0f32.to_le_bytes()); // advance
        assert_eq!(&bytes[2316..2320], &0.0f32.to_le_bytes()); // padding
    }

    // ------------------------------------------------------------------------
    // v0.6.1 (S6.3.3 Multi-outline) tests
    // ------------------------------------------------------------------------

    fn sample_ten() -> OutlineGlyph {
        OutlineGlyph {
            codepoint: '十',
            contours: vec![
                vec![[0.10, 0.45], [0.90, 0.45], [0.90, 0.55], [0.10, 0.55]],
                vec![[0.45, 0.10], [0.55, 0.10], [0.55, 0.90], [0.45, 0.90]],
            ],
            advance: 1.0,
        }
    }

    #[test]
    fn multi_outline_ubo_size_is_fixed_13392_bytes() {
        // v0.6.7: 4 glyph × 384 pt × 8B + 4 × 16 × 16 + 4 × 16 + 16 = 13,392
        assert_eq!(MULTI_OUTLINE_UBO_SIZE, 12_288 + 1_024 + 64 + 16);
        assert_eq!(MULTI_OUTLINE_UBO_SIZE, 13_392);
    }

    #[test]
    fn multi_outline_ubo_fits_in_16kb_webgl2_min_spec() {
        const _: () = assert!(
            MULTI_OUTLINE_UBO_SIZE <= 16 * 1024,
            "MULTI_OUTLINE_UBO_SIZE > 16 KB webgl2 min"
        );
    }

    #[test]
    fn pack_outline_string_empty_text_returns_none() {
        let provider = |_: char| None::<OutlineGlyph>;
        assert!(pack_outline_string("", provider).is_none());
    }

    #[test]
    fn pack_outline_string_layout_is_cumulative() {
        let three = sample_three();
        let ten = sample_ten();
        let provider = move |c: char| match c {
            '三' => Some(three.clone()),
            '十' => Some(ten.clone()),
            _ => None,
        };
        let text = pack_outline_string("三十三", provider).unwrap();
        assert_eq!(text.glyph_count(), 3);
        assert_eq!(text.chunk_count(), 1);
        assert_eq!(text.layout[0].x_offset, 0.0);
        assert_eq!(text.layout[1].x_offset, 1.0);
        assert_eq!(text.layout[2].x_offset, 2.0);
        assert!((text.total_advance - 3.0).abs() < 1e-6);
    }

    #[test]
    fn multi_outline_ubo_chunk_size_is_fixed() {
        let three = sample_three();
        let provider = |_: char| Some(three.clone());
        let text = pack_outline_string("三三三", provider).unwrap();
        let chunks = text.to_std140_ubo_chunks();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), MULTI_OUTLINE_UBO_SIZE);
    }

    #[test]
    fn multi_outline_5_glyphs_splits_to_2_chunks() {
        // v0.6.3: MAX_GLYPHS=4 → 5 chars で 4+1 split。
        let three = sample_three();
        let provider = |_: char| Some(three.clone());
        let text = pack_outline_string("三三三三三", provider).unwrap();
        assert_eq!(text.glyph_count(), 5);
        assert_eq!(text.chunk_count(), 2);
        let chunks = text.to_std140_ubo_chunks();
        assert_eq!(chunks.len(), 2);
        for c in &chunks {
            assert_eq!(c.len(), MULTI_OUTLINE_UBO_SIZE);
        }
    }

    #[test]
    fn multi_outline_4_glyphs_still_single_chunk() {
        // v0.6.3: MAX_GLYPHS=4 境界値、ちょうど 1 chunk 満載。
        let three = sample_three();
        let provider = |_: char| Some(three.clone());
        let text = pack_outline_string("三三三三", provider).unwrap();
        assert_eq!(text.glyph_count(), 4);
        assert_eq!(text.chunk_count(), 1);
    }

    #[test]
    fn multi_outline_first_glyph_points_bit_exact() {
        let three = sample_three();
        let provider = |_: char| Some(three.clone());
        let text = pack_outline_string("三", provider).unwrap();
        let chunk = &text.to_std140_ubo_chunks()[0];
        // First point of first glyph = (0.15, 0.75) at offset 0..8
        assert_eq!(&chunk[0..4], &0.15f32.to_le_bytes());
        assert_eq!(&chunk[4..8], &0.75f32.to_le_bytes());
    }

    #[test]
    fn multi_outline_per_glyph_meta_encodes_advance_contour_count_xoffset() {
        let three = sample_three();
        let ten = sample_ten();
        let provider = move |c: char| match c {
            '三' => Some(three.clone()),
            '十' => Some(ten.clone()),
            _ => None,
        };
        let text = pack_outline_string("三十", provider).unwrap();
        let chunk = &text.to_std140_ubo_chunks()[0];
        // v0.6.7: uGlyphMetas at offset 12,288 + 1,024 = 13,312
        let meta_area = MULTI_OUTLINE_POINTS_AREA_SIZE + MULTI_OUTLINE_CONTOURS_AREA_SIZE;
        assert_eq!(meta_area, 13_312);
        // Glyph 0 (三): advance=1.0, contour_count=3, x_offset=0.0
        assert_eq!(&chunk[meta_area..meta_area + 4], &1.0f32.to_le_bytes());
        assert_eq!(&chunk[meta_area + 4..meta_area + 8], &3.0f32.to_le_bytes());
        assert_eq!(&chunk[meta_area + 8..meta_area + 12], &0.0f32.to_le_bytes());
        // Glyph 1 (十): advance=1.0, contour_count=2, x_offset=1.0 (+16)
        assert_eq!(
            &chunk[meta_area + 16..meta_area + 20],
            &1.0f32.to_le_bytes()
        );
        assert_eq!(
            &chunk[meta_area + 20..meta_area + 24],
            &2.0f32.to_le_bytes()
        );
        assert_eq!(
            &chunk[meta_area + 24..meta_area + 28],
            &1.0f32.to_le_bytes()
        );
    }

    #[test]
    fn multi_outline_utext_meta_encodes_total_advance_and_glyph_count() {
        let three = sample_three();
        let provider = |_: char| Some(three.clone());
        let text = pack_outline_string("三三三", provider).unwrap();
        let chunk = &text.to_std140_ubo_chunks()[0];
        let tm = MULTI_OUTLINE_POINTS_AREA_SIZE
            + MULTI_OUTLINE_CONTOURS_AREA_SIZE
            + MULTI_OUTLINE_METAS_AREA_SIZE;
        assert_eq!(tm, 13_376);
        assert_eq!(&chunk[tm..tm + 4], &3.0f32.to_le_bytes()); // total_advance
        assert_eq!(&chunk[tm + 4..tm + 8], &3.0f32.to_le_bytes()); // chunk_glyph_count
    }
}

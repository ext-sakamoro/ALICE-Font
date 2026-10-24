//! S6.4 v0.7.0 — parametric skeleton shader path (zero-asset browser demo)。
//!
//! v0.6.x outline path が **polygon outline data (L1 provided)** を必要と
//! したのに対し、本 module は **procedurally generated Bezier stroke
//! skeleton** を直接 GPU に届ける。1,053 kanji (KANJI_DB) を CPU で
//! `kanji::build_skeleton(ch)` により法則から生成 → pack → shader が
//! per-pixel でBezier variable-width pen SDF を評価。
//!
//! # Zero-asset payload budget
//!
//! - Skeleton UBO per chunk: **~2,128 byte** (4 glyph × 16 stroke)
//! - 1,053 kanji 全 skeleton (pre-packed): ~540 KB (upper bound)
//! - Shader source: ~5 KB
//! - Radical / IDS DB source (Rust const): shipped in Rust binary (Wasm 化時
//!   に効いてくる、target ~200 KB)
//! - Runtime texture upload: **0 byte** (fragment shader が 数値評価)
//!
//! # L0/L1 分離との関係
//!
//! - **本 module は L0 (ALICE-Font)** = 法則 (radicals + IDS + composition)
//!   から skeleton 生成 → shader 評価 の full path
//! - v0.6.x outline path も L0 だが、data provider (L1) を requires
//! - **v0.7.0 は L0 単独で closure** (原則遵守、法則 shipping 完成)
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use crate::glyph::MAX_GLYPH_STROKES;
use crate::stroke::Stroke;

/// Multi-skeleton UBO の 1 chunk 内 最大 glyph 数。
///
/// v0.7.0: 4 glyph (outline path と同じ、canvas layout uniform 互換維持)。
pub const MULTI_SKELETON_MAX_GLYPHS: usize = 4;

/// Multi-skeleton UBO の per-glyph 最大 stroke 数。
///
/// [`MAX_GLYPH_STROKES`] (= 16) と一致、L0 CPU generator が 16 stroke まで
/// 出力するため。BIZ UDPGothic-comparable coverage (護 = 12 stroke、
/// 議 = 16 stroke = cap)。
pub const MULTI_SKELETON_MAX_STROKES_PER_GLYPH: usize = MAX_GLYPH_STROKES;

/// Stroke ごとに UBO vec4 3 個で pack (p01 / p23 / widths、各 vec4 = 2 point or 2 width + pad)。
///
/// v0.7.2-alpha: 2 → 3 (variable-width pen 対応、width_start / width_end を stroke 単位で追加)。
pub const VEC4_PER_STROKE: usize = 3;

/// Multi-skeleton UBO 内 strokes area 総 byte 数。
///
/// v0.7.2-alpha: 4 glyph × 16 stroke × 3 vec4 × 16 B = **3,072 B** (was 2,048B in v0.7.0-v0.7.1)。
pub const MULTI_SKELETON_STROKES_AREA_SIZE: usize =
    MULTI_SKELETON_MAX_GLYPHS * MULTI_SKELETON_MAX_STROKES_PER_GLYPH * VEC4_PER_STROKE * 16;

/// Multi-skeleton UBO 内 glyph metas area (4 vec4 = **64 B**)。
///
/// Meta layout: (advance, stroke_count_f, x_offset, pen_width)。
pub const MULTI_SKELETON_METAS_AREA_SIZE: usize = MULTI_SKELETON_MAX_GLYPHS * 16;

/// Multi-skeleton UBO 総 byte 数 (fixed per chunk = **3,152 B**、v0.7.2-alpha)。
///
/// Layout (`shaders/analytical_multi_skeleton.frag.glsl` と bit-exact 一致):
/// - Offset 0..3,072: `vec4 uStrokes[192]` (flat 4 glyph × 16 stroke × 3 vec4)
///   - vec4[i*3+0]: (p0.x, p0.y, p1.x, p1.y)
///   - vec4[i*3+1]: (p2.x, p2.y, p3.x, p3.y)
///   - vec4[i*3+2]: (width_start, width_end, 0, 0) ← v0.7.2 追加
/// - Offset 3,072..3,136: `vec4 uGlyphMetas[4]` (advance, stroke_count, x_offset, default_pen)
/// - Offset 3,136..3,152: `vec4 uTextMeta` (total_advance, chunk_glyph_count, 0, 0)
///
/// 3,152 / 16,384 (WebGL2 min UBO) = 19.2% 使用、余裕十分。
pub const MULTI_SKELETON_UBO_SIZE: usize =
    MULTI_SKELETON_STROKES_AREA_SIZE + MULTI_SKELETON_METAS_AREA_SIZE + 16;

/// 1 glyph の pack 済 skeleton (chunk 化前の中間表現)。
#[derive(Debug, Clone, Copy)]
pub struct SkeletonGlyph {
    pub codepoint: char,
    pub strokes: [Stroke; MAX_GLYPH_STROKES],
    /// v0.7.2-alpha: per-stroke variable-width pen (start / end)。
    /// Default = `(pen_width, pen_width)` (uniform)、v0.7.2 で書道 taper 表現。
    /// Shader が Bezier t で `mix(start, end, t)` 補間。
    pub stroke_widths: [(f32, f32); MAX_GLYPH_STROKES],
    pub stroke_count: usize,
    pub advance: f32,
    /// Default pen radius (em 単位、`stroke_widths` が uniform=0.0 時の fallback)。
    pub pen_width: f32,
}

/// 単一 glyph の text layout 内配置情報 (em 座標)。
#[derive(Debug, Clone, Copy)]
pub struct SkeletonGlyphLayout {
    pub codepoint: char,
    pub x_offset: f32,
}

/// Multi-skeleton payload — N glyph を chunk 分割して GPU に届ける。
pub struct MultiSkeletonGlyphPayload {
    pub glyphs: Vec<SkeletonGlyph>,
    pub layout: Vec<SkeletonGlyphLayout>,
    pub total_advance: f32,
}

impl MultiSkeletonGlyphPayload {
    #[must_use]
    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }

    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.glyphs.len().div_ceil(MULTI_SKELETON_MAX_GLYPHS)
    }

    /// 全 chunk を std140 UBO byte 列に変換 (各 chunk = `MULTI_SKELETON_UBO_SIZE` byte)。
    #[must_use]
    pub fn to_std140_ubo_chunks(&self) -> Vec<Vec<u8>> {
        let mut chunks = Vec::with_capacity(self.chunk_count());

        for chunk_idx in 0..self.chunk_count() {
            let start = chunk_idx * MULTI_SKELETON_MAX_GLYPHS;
            let end = (start + MULTI_SKELETON_MAX_GLYPHS).min(self.glyphs.len());
            let chunk_glyph_count = end - start;
            let mut out = Vec::with_capacity(MULTI_SKELETON_UBO_SIZE);

            // Section 1: strokes (4 glyph × 16 stroke × 2 vec4)
            for glyph_idx in 0..MULTI_SKELETON_MAX_GLYPHS {
                if glyph_idx < chunk_glyph_count {
                    write_glyph_strokes_padded(&mut out, &self.glyphs[start + glyph_idx]);
                } else {
                    // Zero-fill padding
                    let zero_size = MULTI_SKELETON_MAX_STROKES_PER_GLYPH * VEC4_PER_STROKE * 16;
                    out.extend_from_slice(&vec![0u8; zero_size]);
                }
            }

            // Section 2: glyph metas
            for glyph_idx in 0..MULTI_SKELETON_MAX_GLYPHS {
                if glyph_idx < chunk_glyph_count {
                    let g = &self.glyphs[start + glyph_idx];
                    let x_off = self.layout[start + glyph_idx].x_offset;
                    let stroke_count_f = g.stroke_count.min(MAX_GLYPH_STROKES) as f32;
                    out.extend_from_slice(&g.advance.to_le_bytes());
                    out.extend_from_slice(&stroke_count_f.to_le_bytes());
                    out.extend_from_slice(&x_off.to_le_bytes());
                    out.extend_from_slice(&g.pen_width.to_le_bytes());
                } else {
                    out.extend_from_slice(&[0u8; 16]);
                }
            }

            // Section 3: uTextMeta
            out.extend_from_slice(&self.total_advance.to_le_bytes());
            out.extend_from_slice(&(chunk_glyph_count as f32).to_le_bytes());
            out.extend_from_slice(&[0u8; 8]); // padding to vec4

            debug_assert_eq!(out.len(), MULTI_SKELETON_UBO_SIZE);
            chunks.push(out);
        }

        chunks
    }
}

/// Per-glyph stroke pack (padded to `MAX_GLYPH_STROKES`、each stroke = 3 vec4)。
///
/// v0.7.2-alpha: 3rd vec4 = (width_start, width_end, 0, 0) を追加。
/// Default pen width が `stroke_widths` に (0, 0) の場合、shader 側で
/// `default_pen` (uGlyphMetas.w) を代替使用。
fn write_glyph_strokes_padded(out: &mut Vec<u8>, glyph: &SkeletonGlyph) {
    for stroke_idx in 0..MULTI_SKELETON_MAX_STROKES_PER_GLYPH {
        if stroke_idx < glyph.stroke_count {
            let s = &glyph.strokes[stroke_idx];
            let (w_start, w_end) = glyph.stroke_widths[stroke_idx];
            // vec4 #0: (p0.x, p0.y, p1.x, p1.y)
            out.extend_from_slice(&s.p0.x.to_le_bytes());
            out.extend_from_slice(&s.p0.y.to_le_bytes());
            out.extend_from_slice(&s.p1.x.to_le_bytes());
            out.extend_from_slice(&s.p1.y.to_le_bytes());
            // vec4 #1: (p2.x, p2.y, p3.x, p3.y)
            out.extend_from_slice(&s.p2.x.to_le_bytes());
            out.extend_from_slice(&s.p2.y.to_le_bytes());
            out.extend_from_slice(&s.p3.x.to_le_bytes());
            out.extend_from_slice(&s.p3.y.to_le_bytes());
            // vec4 #2: (width_start, width_end, 0, 0)
            out.extend_from_slice(&w_start.to_le_bytes());
            out.extend_from_slice(&w_end.to_le_bytes());
            out.extend_from_slice(&[0u8; 8]);
        } else {
            // Zero-fill 3 vec4 padding
            out.extend_from_slice(&[0u8; 48]);
        }
    }
}

/// Text 文字列 → `MultiSkeletonGlyphPayload`。provider = ch → `SkeletonGlyph`。
///
/// # Returns
/// - `Some(payload)` — 少なくとも 1 char が provider により生成された
/// - `None` — 全 char が unregistered (空 text 含む)
#[must_use]
pub fn pack_skeleton_string<F>(text: &str, provider: F) -> Option<MultiSkeletonGlyphPayload>
where
    F: Fn(char) -> Option<SkeletonGlyph>,
{
    let mut glyphs = Vec::new();
    let mut layout = Vec::new();
    let mut cursor_x = 0.0f32;

    for ch in text.chars() {
        if let Some(g) = provider(ch) {
            layout.push(SkeletonGlyphLayout {
                codepoint: ch,
                x_offset: cursor_x,
            });
            cursor_x += g.advance;
            glyphs.push(g);
        }
    }

    if glyphs.is_empty() {
        return None;
    }

    Some(MultiSkeletonGlyphPayload {
        glyphs,
        layout,
        total_advance: cursor_x,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stroke::Point2;

    fn sample_stroke(x0: f32, y0: f32, x1: f32, y1: f32) -> Stroke {
        Stroke::line(Point2::new(x0, y0), Point2::new(x1, y1))
    }

    fn sample_skeleton_ichi() -> SkeletonGlyph {
        let mut strokes = [Stroke::line(Point2::ZERO, Point2::ZERO); MAX_GLYPH_STROKES];
        strokes[0] = sample_stroke(0.1, 0.5, 0.9, 0.5);
        SkeletonGlyph {
            codepoint: '一',
            strokes,
            stroke_widths: [(0.05, 0.05); MAX_GLYPH_STROKES],
            stroke_count: 1,
            advance: 1.0,
            pen_width: 0.05,
        }
    }

    fn sample_skeleton_san() -> SkeletonGlyph {
        let mut strokes = [Stroke::line(Point2::ZERO, Point2::ZERO); MAX_GLYPH_STROKES];
        strokes[0] = sample_stroke(0.15, 0.75, 0.85, 0.75);
        strokes[1] = sample_stroke(0.20, 0.50, 0.80, 0.50);
        strokes[2] = sample_stroke(0.10, 0.20, 0.90, 0.20);
        SkeletonGlyph {
            codepoint: '三',
            strokes,
            stroke_widths: [(0.05, 0.05); MAX_GLYPH_STROKES],
            stroke_count: 3,
            advance: 1.0,
            pen_width: 0.05,
        }
    }

    #[test]
    fn multi_skeleton_ubo_size_is_fixed_3152_bytes() {
        // v0.7.2-alpha: 4 glyph × 16 stroke × 3 vec4 × 16 + 4 × 16 + 16 = 3,152
        assert_eq!(MULTI_SKELETON_UBO_SIZE, 3_072 + 64 + 16);
        assert_eq!(MULTI_SKELETON_UBO_SIZE, 3_152);
    }

    #[test]
    fn multi_skeleton_ubo_fits_in_16kb_webgl2_min() {
        const _: () = assert!(
            MULTI_SKELETON_UBO_SIZE <= 16 * 1024,
            "MULTI_SKELETON_UBO_SIZE > 16 KB webgl2 min"
        );
    }

    #[test]
    fn pack_skeleton_string_empty_returns_none() {
        let provider = |_: char| None::<SkeletonGlyph>;
        assert!(pack_skeleton_string("", provider).is_none());
    }

    #[test]
    fn pack_skeleton_string_layout_is_cumulative() {
        let ichi = sample_skeleton_ichi();
        let san = sample_skeleton_san();
        let provider = move |c: char| match c {
            '一' => Some(ichi),
            '三' => Some(san),
            _ => None,
        };
        let text = pack_skeleton_string("一三一", provider).unwrap();
        assert_eq!(text.glyph_count(), 3);
        assert_eq!(text.chunk_count(), 1);
        assert!((text.total_advance - 3.0).abs() < 1e-6);
        assert_eq!(text.layout[0].x_offset, 0.0);
        assert_eq!(text.layout[1].x_offset, 1.0);
        assert_eq!(text.layout[2].x_offset, 2.0);
    }

    #[test]
    fn multi_skeleton_ubo_chunk_size_fixed() {
        let ichi = sample_skeleton_ichi();
        let provider = |_: char| Some(ichi);
        let text = pack_skeleton_string("一一一", provider).unwrap();
        let chunks = text.to_std140_ubo_chunks();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), MULTI_SKELETON_UBO_SIZE);
    }

    #[test]
    fn multi_skeleton_5_glyphs_splits_to_2_chunks() {
        // v0.7.0: MAX_GLYPHS = 4 → 5 char で 4+1 split
        let ichi = sample_skeleton_ichi();
        let provider = |_: char| Some(ichi);
        let text = pack_skeleton_string("一一一一一", provider).unwrap();
        assert_eq!(text.glyph_count(), 5);
        assert_eq!(text.chunk_count(), 2);
        let chunks = text.to_std140_ubo_chunks();
        assert_eq!(chunks.len(), 2);
        for c in &chunks {
            assert_eq!(c.len(), MULTI_SKELETON_UBO_SIZE);
        }
    }

    #[test]
    fn multi_skeleton_first_stroke_bit_exact() {
        let ichi = sample_skeleton_ichi();
        let provider = |_: char| Some(ichi);
        let text = pack_skeleton_string("一", provider).unwrap();
        let chunk = &text.to_std140_ubo_chunks()[0];
        // First stroke of 一: p0=(0.1, 0.5), p1=(0.5, 0.5) (line degenerate)
        assert_eq!(&chunk[0..4], &0.1f32.to_le_bytes());
        assert_eq!(&chunk[4..8], &0.5f32.to_le_bytes());
    }

    #[test]
    fn multi_skeleton_glyph_meta_encodes_advance_stroke_count() {
        let san = sample_skeleton_san();
        let provider = |_: char| Some(san);
        let text = pack_skeleton_string("三", provider).unwrap();
        let chunk = &text.to_std140_ubo_chunks()[0];
        // v0.7.2-alpha: uGlyphMetas at offset 3,072 (was 2,048 in v0.7.0)
        assert_eq!(&chunk[3072..3076], &1.0f32.to_le_bytes()); // advance
        assert_eq!(&chunk[3076..3080], &3.0f32.to_le_bytes()); // stroke_count
        assert_eq!(&chunk[3080..3084], &0.0f32.to_le_bytes()); // x_offset
        assert_eq!(&chunk[3084..3088], &0.05f32.to_le_bytes()); // pen_width
    }

    #[test]
    fn multi_skeleton_utext_meta_encodes_total_advance() {
        let ichi = sample_skeleton_ichi();
        let provider = |_: char| Some(ichi);
        let text = pack_skeleton_string("一一", provider).unwrap();
        let chunk = &text.to_std140_ubo_chunks()[0];
        // v0.7.2-alpha: uTextMeta at offset 3,072 + 64 = 3,136 (was 2,112 in v0.7.0)
        assert_eq!(&chunk[3136..3140], &2.0f32.to_le_bytes()); // total_advance
        assert_eq!(&chunk[3140..3144], &2.0f32.to_le_bytes()); // chunk_glyph_count
    }

    #[test]
    fn multi_skeleton_stroke_width_bit_exact() {
        // v0.7.2-alpha: 3rd vec4 per stroke = (w_start, w_end, 0, 0) at offset 32..48
        let mut strokes = [Stroke::line(Point2::ZERO, Point2::ZERO); MAX_GLYPH_STROKES];
        strokes[0] = sample_stroke(0.1, 0.5, 0.9, 0.5);
        let mut widths = [(0.05, 0.05); MAX_GLYPH_STROKES];
        widths[0] = (0.03, 0.08); // taper from thin to thick
        let g = SkeletonGlyph {
            codepoint: '一',
            strokes,
            stroke_widths: widths,
            stroke_count: 1,
            advance: 1.0,
            pen_width: 0.05,
        };
        let provider = |_: char| Some(g);
        let text = pack_skeleton_string("一", provider).unwrap();
        let chunk = &text.to_std140_ubo_chunks()[0];
        // Stroke 0's width vec4 at offset 32 (skip 2 vec4 = 32B for positions)
        assert_eq!(&chunk[32..36], &0.03f32.to_le_bytes()); // width_start
        assert_eq!(&chunk[36..40], &0.08f32.to_le_bytes()); // width_end
    }
}

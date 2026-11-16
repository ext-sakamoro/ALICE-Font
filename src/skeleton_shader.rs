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

/// The vertex shader (GLSL ES 3.00, WebGL2) that reads [`MultiSkeletonGlyphPayload::to_std140_ubo_chunks`]:
/// one instance per glyph, the text's em box fitted into the canvas with its aspect kept
///
/// Uniforms: the `MultiSkeletonBlock` block, `uCanvasSize` (pixels) and `uYPadding` (em above and
/// below the baseline box); attribute `aQuadPos` (the unit quad, -1..1)
pub const MULTI_SKELETON_VERT_GLSL: &str =
    include_str!("../shaders/analytical_multi_skeleton.vert.glsl");

/// The fragment shader (GLSL ES 3.00, WebGL2) paired with [`MULTI_SKELETON_VERT_GLSL`]: the distance
/// to each stroke's cubic (sampled), the pen width interpolated along the stroke, and
/// `1 - smoothstep(pen - aa, pen + aa, d)` for the coverage that mixes `uBgColor` to `uFgColor`
pub const MULTI_SKELETON_FRAG_GLSL: &str = concat!(
    "#version 300 es\n",
    "precision highp float;\n",
    // without it a fragment stage's `int` is mediump: `float(i)` of a loop index or a count can then be
    // computed in half precision (measured on one GPU: 4-18/255 off the CPU reference)
    "precision highp int;\n",
    include_str!("../shaders/skeleton_distance.glsl"),
    include_str!("../shaders/analytical_multi_skeleton.frag.glsl"),
);

/// The skeleton as a distance, for a shader to include (GLSL ES 3.00, with no `#version` and no `precision`: the
/// including shader states both): the `MultiSkeletonBlock` UBO block, `skeletonStrokeCount(glyph)`,
/// `skeletonStroke(p, glyph, s)` (the distance to stroke `s`'s centre line and its pen half-width there) and
/// `skeletonDistance(p, glyph)` (the glyph's signed distance: negative inside its strokes). The text shader
/// [`MULTI_SKELETON_FRAG_GLSL`] is built on it; [`skeleton_stroke`] and [`skeleton_distance`] are its Rust twins
pub const SKELETON_DISTANCE_GLSL: &str = include_str!("../shaders/skeleton_distance.glsl");

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

impl SkeletonGlyph {
    /// The packed form of a glyph's skeleton (a [`GlyphSkeleton`](crate::glyph::GlyphSkeleton) of the glyph
    /// dispatcher: ASCII, kana or kanji), every stroke drawn with the pen half-width `pen` (em). Strokes past
    /// [`MAX_GLYPH_STROKES`] are not kept (the skeleton has no more)
    #[must_use]
    pub fn from_skeleton(
        codepoint: char,
        skeleton: &crate::glyph::GlyphSkeleton,
        pen: f32,
    ) -> Self {
        let count = skeleton.stroke_count.min(MAX_GLYPH_STROKES);
        let mut strokes = [Stroke::line(crate::stroke::Point2::ZERO, crate::stroke::Point2::ZERO);
            MAX_GLYPH_STROKES];
        strokes[..count].copy_from_slice(&skeleton.strokes[..count]);
        Self {
            codepoint,
            strokes,
            stroke_widths: [(pen, pen); MAX_GLYPH_STROKES],
            stroke_count: count,
            advance: skeleton.advance,
            pen_width: pen,
        }
    }
}

/// A provider for [`pack_skeleton_string`] over every character the glyph dispatcher has a skeleton for (ASCII,
/// kana, kanji), each stroke drawn with the font's own pen for `params` ([`MetaFontParams::stroke_half_width`],
/// the half-width the weight gives). `None` for a character with no skeleton
///
/// [`MetaFontParams::stroke_half_width`]: crate::param::MetaFontParams::stroke_half_width
pub fn skeleton_lookup(
    params: &crate::param::MetaFontParams,
) -> impl Fn(char) -> Option<SkeletonGlyph> + '_ {
    move |ch| {
        let skeleton = crate::glyph::dispatcher::skeleton(ch, params)?;
        Some(SkeletonGlyph::from_skeleton(
            ch,
            &skeleton,
            params.stroke_half_width(),
        ))
    }
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

    #[test]
    fn shipped_shader_text_matches_the_ubo_layout() {
        for src in [MULTI_SKELETON_VERT_GLSL, MULTI_SKELETON_FRAG_GLSL] {
            assert!(src.starts_with("#version 300 es"));
            assert!(src.contains("layout(std140) uniform MultiSkeletonBlock"));
            assert!(src.contains(&alloc::format!(
                "const int MAX_GLYPHS = {MULTI_SKELETON_MAX_GLYPHS};"
            )));
            assert!(src.contains(&alloc::format!(
                "const int MAX_STROKES_PER_GLYPH = {MULTI_SKELETON_MAX_STROKES_PER_GLYPH};"
            )));
            assert!(src.contains(&alloc::format!(
                "const int VEC4_PER_STROKE = {VEC4_PER_STROKE};"
            )));
        }
        assert!(MULTI_SKELETON_VERT_GLSL.contains("gl_InstanceID"));
        assert!(MULTI_SKELETON_FRAG_GLSL.contains("smoothstep(penW - aa, penW + aa, d)"));
    }
}

// ── The skeleton as a distance: the Rust twin of shaders/skeleton_distance.glsl ──────────────────────────────────
//
// The same operations in the same order as the GLSL, on the same std140 numbers the GPU reads, so the field a CPU
// reads (a drop landing in a groove, a ball rolling into it) is the field the GPU draws. GLSL evaluates `a * b * c`
// left to right and defines `mix(x, y, a)` as `x * (1 - a) + y * a`; both are written out that way here. The square
// root is alice-det-math's (correctly rounded); a GPU's `length` and division are within the precision GLSL ES
// allows, so the two agree to a stated tolerance, not bit for bit (tests/skeleton_distance.rs).

const BEZIER_SAMPLES: usize = 12;

fn read_f32(chunk: &[u8], vec4: usize, lane: usize) -> f32 {
    let at = vec4 * 16 + lane * 4;
    f32::from_le_bytes([chunk[at], chunk[at + 1], chunk[at + 2], chunk[at + 3]])
}

/// GLSL `mix(x, y, a)` as the GLSL ES 3.00 spec defines it
fn glsl_mix(x: f32, y: f32, a: f32) -> f32 {
    x * (1.0 - a) + y * a
}

fn glsl_length(x: f32, y: f32) -> f32 {
    alice_det_math::ops::sqrt(x * x + y * y)
}

fn bezier_at(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    let u2 = u * u;
    let u3 = u2 * u;
    let t2 = t * t;
    let t3 = t2 * t;
    // `u3 * p0 + 3.0 * u2 * t * p1 + 3.0 * u * t2 * p2 + t3 * p3`, each product left to right
    let a1 = 3.0 * u2 * t;
    let a2 = 3.0 * u * t2;
    [
        u3 * p0[0] + a1 * p1[0] + a2 * p2[0] + t3 * p3[0],
        u3 * p0[1] + a1 * p1[1] + a2 * p2[1] + t3 * p3[1],
    ]
}

/// (the least distance from `p` to the stroke's 12-piece polyline, the Bezier t of that closest point)
fn sd_bezier_approx_t(
    p: [f32; 2],
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
) -> (f32, f32) {
    let mut min_d = 1e10_f32;
    let mut best_t = 0.0_f32;
    let mut prev = p0;
    for i in 1..=BEZIER_SAMPLES {
        let ti = i as f32 / BEZIER_SAMPLES as f32;
        let curr = bezier_at(p0, p1, p2, p3, ti);
        let ba = [curr[0] - prev[0], curr[1] - prev[1]];
        let pa = [p[0] - prev[0], p[1] - prev[1]];
        let ba_len2 = ba[0] * ba[0] + ba[1] * ba[1];
        let mut seg_t = 0.0_f32;
        let d = if ba_len2 < 1e-12 {
            glsl_length(pa[0], pa[1])
        } else {
            seg_t = crate::law_float::law_clamp_f32(
                (pa[0] * ba[0] + pa[1] * ba[1]) / ba_len2,
                0.0,
                1.0,
            );
            glsl_length(pa[0] - ba[0] * seg_t, pa[1] - ba[1] * seg_t)
        };
        if d < min_d {
            min_d = d;
            let t_prev = (i - 1) as f32 / BEZIER_SAMPLES as f32;
            best_t = glsl_mix(t_prev, ti, seg_t);
        }
        prev = curr;
    }
    (min_d, best_t)
}

/// How many strokes glyph `glyph` of a std140 chunk has, as the GPU reads it (`int(uGlyphMetas[glyph].y)`); `None`
/// for a chunk that is not [`MULTI_SKELETON_UBO_SIZE`] bytes or a glyph past [`MULTI_SKELETON_MAX_GLYPHS`]
#[must_use]
pub fn skeleton_stroke_count(chunk: &[u8], glyph: usize) -> Option<usize> {
    if chunk.len() != MULTI_SKELETON_UBO_SIZE || glyph >= MULTI_SKELETON_MAX_GLYPHS {
        return None;
    }
    let metas = MULTI_SKELETON_STROKES_AREA_SIZE / 16;
    // GLSL `int(f)` truncates toward zero; a negative or NaN count draws nothing
    let n = read_f32(chunk, metas + glyph, 1);
    Some(if n >= 1.0 { n as usize } else { 0 })
}

/// Stroke `stroke` of glyph `glyph` of a std140 chunk (one of [`MultiSkeletonGlyphPayload::to_std140_ubo_chunks`]),
/// seen from `p` (the glyph's em units): `[the distance to its centre line, its pen half-width there]`, as
/// `skeletonStroke` computes it. `None` for a malformed chunk or an index past the chunk's glyphs or strokes
#[must_use]
pub fn skeleton_stroke(chunk: &[u8], glyph: usize, stroke: usize, p: [f32; 2]) -> Option<[f32; 2]> {
    if chunk.len() != MULTI_SKELETON_UBO_SIZE
        || glyph >= MULTI_SKELETON_MAX_GLYPHS
        || stroke >= MULTI_SKELETON_MAX_STROKES_PER_GLYPH
    {
        return None;
    }
    let metas = MULTI_SKELETON_STROKES_AREA_SIZE / 16;
    let default_pen = read_f32(chunk, metas + glyph, 3);
    let base =
        glyph * MULTI_SKELETON_MAX_STROKES_PER_GLYPH * VEC4_PER_STROKE + stroke * VEC4_PER_STROKE;
    let p0 = [read_f32(chunk, base, 0), read_f32(chunk, base, 1)];
    let p1 = [read_f32(chunk, base, 2), read_f32(chunk, base, 3)];
    let p2 = [read_f32(chunk, base + 1, 0), read_f32(chunk, base + 1, 1)];
    let p3 = [read_f32(chunk, base + 1, 2), read_f32(chunk, base + 1, 3)];
    let mut w_start = read_f32(chunk, base + 2, 0);
    let mut w_end = read_f32(chunk, base + 2, 1);
    if w_start < 1e-6 {
        w_start = default_pen;
    }
    if w_end < 1e-6 {
        w_end = default_pen;
    }
    let (d, t) = sd_bezier_approx_t(p, p0, p1, p2, p3);
    Some([d, glsl_mix(w_start, w_end, t)])
}

/// Glyph `glyph`'s signed distance at `p` (em units), as `skeletonDistance` computes it: the least over its strokes
/// of the distance to the centre line less the pen there, negative inside; `1e10` for a glyph with no strokes.
/// `None` for a malformed chunk or a glyph past [`MULTI_SKELETON_MAX_GLYPHS`]
#[must_use]
pub fn skeleton_distance(chunk: &[u8], glyph: usize, p: [f32; 2]) -> Option<f32> {
    let count = skeleton_stroke_count(chunk, glyph)?.min(MULTI_SKELETON_MAX_STROKES_PER_GLYPH);
    let mut best = 1e10_f32;
    for s in 0..count {
        let [d, pen] = skeleton_stroke(chunk, glyph, s, p)?;
        best = crate::law_float::law_min_f32(best, d - pen);
    }
    Some(best)
}

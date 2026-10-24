# ALICE-Font

[English](README.md) · [日本語](README.ja.md)

**Procedural CJK font — no atlas, no TTF, no bitmap.**

> "Don't ship data. Ship laws." — the ALICE principle

ALICE-Font follows the ALICE core principle 「データを送るな、法則を送れ」
(don't ship data — ship laws). Every glyph is composed
**procedurally at runtime** from a tiny set of laws: a 40-byte
`MetaFontParams`, a library of parametric radical stroke skeletons,
and IDS composition rules describing how each kanji decomposes into
radicals. The runtime evaluates those laws for every pixel; no
outline table is embedded, no SDF atlas is baked, no `.ttf` /
`.otf` is shipped.

## Why this matters

Every prior "efficient font" solution — MSDF atlases, TinySDF,
Slug, WOFF2, variable fonts — still ships **data**: pixel textures,
bezier outlines, glyph tables. ALICE-Font ships **laws**:
mathematical rules that reconstruct any glyph on demand. The
resulting payload is measured in kilobytes and every glyph is
resolution-independent by construction.

## What you ship

| Primitive | Payload (target at S6 completion) | Role |
|-----------|-----------------------------------|------|
| `MetaFontParams` | **40 bytes** (10 × f32) | Style: weight, width, serif, contrast, slant, metrics, roundness |
| Radical stroke library | ~150 KB Rust const | 214 Kang Xi radicals as parametric Bezier skeletons |
| IDS composition rules | ~30 KB | How each kanji decomposes into radicals (⿰⿱⿲⿴ …) |
| Runtime pen model | ~5 KB code | Variable-width Bezier pen driven by `MetaFontParams` |
| Layout engine | ~5 KB code | IDS operator → radical placement transforms |
| **Total shipped for a full 常用漢字 2,136 renderer** | **≈ 200 KB** | — |

Compare: a full CJK TTF (Noto Sans JP) is ~7 MB just for one weight.

## The three primitives, in Rust

```rust
use alice_font::{MetaFontParams, glyph_dispatcher};

// 40-byte parametric style
let regular = MetaFontParams::sans_regular();

// A hiragana glyph is composed from parametric strokes, no texture
let sdf = glyph_dispatcher::generate('あ', &regular);

// The same 40 bytes, morphed
let bolder = MetaFontParams { weight: 0.18, ..regular };
let sdf_bold = glyph_dispatcher::generate('あ', &bolder);
```

For CJK, the same call routes through `cjk::ids_db` +
`cjk::radicals` + `cjk::layout` — the character's IDS decomposition
is looked up, each radical's stroke skeleton is scaled and placed,
and the union is returned as a `GlyphSdf`.

## What is *not* here (deliberately)

- **No embedded TTF outline table.** A previous version (`v0.3.x`)
  shipped ~1 MB of BIZ UDPGothic cubic-Bezier outline coordinates
  as a Rust const. That was data smuggled through the crate
  boundary; it violated the ALICE principle. Removed in `v0.4.0`.
  The historical implementation is preserved on the
  [`archive/sdf-atlas-approach`](https://github.com/ext-sakamoro/ALICE-Font/tree/archive/sdf-atlas-approach)
  branch.
- **No SDF atlas generator.** The prior `tools/alice-font-atlas`
  CLI (TTF → multi-page SDF atlas) is also on the archive branch.
  It solved a different problem (compress a *given* font into a
  texture); ALICE-Font's problem is to reconstruct a font from
  laws with no source TTF at all.
- **No `.ttf` / `.otf` runtime dependency.** The crate exposes
  procedural generators only.

## Modules

| Module | Description |
|--------|-------------|
| [`param`] | `MetaFontParams` — 40-byte parametric style descriptor |
| [`stroke`] | Variable-width pen model applied along Bezier skeleton curves |
| [`glyph`] | SDF glyph procedural generation — ASCII, Hiragana, Katakana, CJK stroke primitives |
| [`cjk`] | IDS parser, radical library, composition layout, IDS DB |
| [`atlas`] | Runtime GPU-friendly SDF tile cache (derived from procedural output; not a shipped asset) |
| [`shaper`] | Text shaper — kerning, horizontal advance, line layout |
| [`game`] | `TextEffect` — canonical spec for outline / shadow / glow / gradient / dissolve / … |
| [`outline`] | `OutlineGlyph` polygon contours — shader packing, and `rasterize` → `Result<GlyphSdf, RasterizeError>` (exact signed distance to the non-zero-winding region, tile `TILE_EM_*`; refuses non-finite input and outlines over `RASTER_MAX_*`) |
| [`license`] | Font-license tracking (per-title, platform, seat-limit, 32-byte wire format) |
| [`sdf_bridge`] (feature `sdf`) | `Stroke` / `GlyphSkeleton` → `alice_sdf::Sdf2dNode` parametric adapter (Milestone A.3、2026-08-06) |

### `sdf` feature — ALICE-SDF bridge (Milestone A.3、2026-08-06)

`Stroke` / `GlyphSkeleton` (parametric Bezier stroke) を `alice_sdf::Sdf2dNode::Bezier` に変換する adapter。**GlyphSdf (64x64 bitmap) ではなく parametric な skeleton path を SDF に落とす** = 「データを送るな、法則を送れ」原則準拠の law-only path。

```rust
use alice_font::sdf_bridge::{stroke_to_sdf2d, glyph_skeleton_to_sdf2d};
use alice_font::stroke::{Point2, Stroke};

let stroke = Stroke::new(
    Point2::new(0.0, 0.0), Point2::new(0.3, 0.5),
    Point2::new(0.7, 0.5), Point2::new(1.0, 0.0),
);
let sdf = stroke_to_sdf2d(&stroke, 0.05);
// sdf = alice_sdf::Sdf2dNode::Bezier { p0, p1, p2, p3, thickness: 0.05 }
```

path dep のみ (alice-sdf は crates.io 未公開)、`Cargo.toml` で `features = ["sdf"]` で enable。

## Current status (v0.7.0-alpha)

- **S6.2.1 完了** — 教育漢字 grade 1-6 全 1,026 IDS 完了 (v0.4.6)
- **S6.3.1 完了** — fragment shader analytical Bezier distance PoC (v0.5.0)
- **S6.3.2 完了** — instanced multi-glyph text rendering + UBO packing (v0.5.4)
- **S6.3.3 完了** — polygon outline shader path (v0.6.0-v0.6.7):
  - Winding-number test in fragment shader, zero texture upload
  - **384 pt × 4 glyph × 16 contour** per chunk, 13,392B UBO
  - Aspect-preserving canvas fit + descender Y padding (0.15 em)
  - L0/L1 分離: L0 = shader path only, L1 (outline provider) = BIZ UDPGothic
    outline data provider (JIS X 0208 Level 1, 3,344 chars)
- **S6.4 infrastructure** — parametric skeleton shader path (v0.7.0-alpha):
  - L0 単独完結 (radicals + IDS → Bezier skeleton → shader)
  - Zero-asset ~76 KB total payload (shader + UBO + HTML)
  - Content coverage limited (73 / 1,053 KANJI_DB real render、残り placeholder)

## License

`LicenseRef-Proprietary`. The text is [LICENSE-Proprietary](LICENSE-Proprietary).

Enquiries: https://extoria.co.jp/

## Author

Moroya Sakamoto

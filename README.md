# ALICE-Font

**Parametric Metafont Renderer — 40-byte parameters → infinite-resolution SDF glyphs**

> "Don't send fonts. Send the formula."

## The Problem

The modern web downloads **2-5 MB of font files** (TTF/WOFF2) just to display text. Every weight, every style, every language subset is a separate binary blob of pre-drawn outlines.

| Font | WOFF2 Size | ALICE Params | Reduction |
|------|-----------|-------------|-----------|
| Noto Sans Regular | 550 KB | **40 bytes** | **14,000x** |
| Noto Sans (9 weights) | 4.5 MB | **360 bytes** | **12,500x** |
| Full CJK family | 15 MB | **40 bytes + rules** | **~100,000x** |

## The Solution

ALICE-Font replaces font files with **10 floating-point parameters (40 bytes)** that procedurally generate SDF glyphs at any resolution:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  40 bytes   │────▶│ ALICE-Font  │────▶│  SDF Atlas  │────▶│    GPU      │
│  MetaParams │     │ Stroke gen  │     │  LRU cache  │     │  Rendering  │
│  (network)  │     │  Procedural │     │  tile-based │     │  Infinite   │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

## Metafont Parameters (40 bytes)

```rust
pub struct MetaFontParams {
    pub weight: f32,       // Stroke width (0.0=hairline, 1.0=black)
    pub width: f32,        // Horizontal stretch (0.5=condensed, 1.5=extended)
    pub serif: f32,        // Serif amount (0.0=sans, 1.0=full serif)
    pub contrast: f32,     // Thick/thin ratio (0.0=mono, 1.0=high contrast)
    pub slant: f32,        // Italic angle in radians
    pub x_height: f32,     // x-height ratio (0.4-0.6 typical)
    pub cap_height: f32,   // Cap height ratio
    pub ascender: f32,     // Ascender ratio
    pub descender: f32,    // Descender ratio
    pub roundness: f32,    // Corner rounding (0.0=sharp, 1.0=circular)
}
```

## Architecture

- **`param`** — MetaFontParams definition and preset fonts
- **`stroke`** — Parametric stroke model (Bezier skeleton + variable-width pen)
- **`glyph`** — SDF glyph procedural generation from stroke skeletons
  - `glyph::dispatcher` — Unicode-aware routing (ASCII / Hiragana / Katakana / Kanji)
  - `glyph::hiragana` — 82 hiragana characters (清音 + 濁音 + 半濁音 + 小書き)
  - `glyph::katakana` — 83 katakana characters
  - `glyph::kanji` — IDS-driven kanji composition engine (160+ components)
- **`cjk`** — CJK support
  - `cjk::layout::CompositionLayout` — IDS 12 operators (`⿰⿱⿲⿳⿴⿵⿶⿷⿸⿹⿺⿻`)
  - `cjk::radicals::RADICALS` — full Kangxi 214 radical table
  - `cjk::ids` — IDS parser (`⿰日月` → tree)
  - `cjk::ids_db` — pre-defined kanji IDS definitions
- **`atlas`** — SDF atlas management (LRU cache, GPU tile layout)
  - `SdfAtlas` — single-page atlas (legacy, ASCII-focused)
  - `SdfAtlasMulti` — multi-page atlas for CJK (up to 32,768 glyphs)
- **`shaper`** — Text shaping (kerning, line layout, paragraph composition)

## Quick Start

### ASCII glyph (v0.1.0 path)

```rust
use alice_font::{MetaFontParams, GlyphGenerator};

let params = MetaFontParams::sans_regular();
let gen = GlyphGenerator::new(&params);
let sdf = gen.generate(b'A');
let dist = sdf.sample(0.5, 0.5); // negative inside the glyph
```

### CJK (v0.2.0+) via SdfAtlasMulti

```rust
use alice_font::{MetaFontParams, SdfAtlasMulti};

let mut atlas = SdfAtlasMulti::new(3, 32, MetaFontParams::sans_regular());

// ASCII, hiragana, katakana, and kanji all dispatch through the same API.
let chars: Vec<char> = "Hello, 株式会社エクストーリア。"
    .chars()
    .collect();
atlas.preload(&chars);

// `atlas.peek('明')` → AtlasEntryMulti { page_id, uv_*, advance, ... }
// Upload `atlas.page_pixels(p)` to a GL_TEXTURE_2D_ARRAY layer per page.
```

### IDS parser

```rust
use alice_font::cjk::ids::{parse, Ids};
use alice_font::cjk::layout::CompositionLayout;

let tree = parse("⿰日月").unwrap();  // 明 = day + moon
match tree {
    Ids::Binary { layout, .. } => assert_eq!(layout, CompositionLayout::LeftRight),
    _ => unreachable!(),
}
```

## Mathematical Foundation

### Parametric Stroke Model

Each glyph is defined by a **skeleton** (Bezier curves) and a **pen model** (variable-width ellipse):

```
stroke(t) = skeleton(t) ± pen_width(t) × normal(t)
```

The pen width varies along the stroke based on the `contrast` parameter:

```
pen_width(t) = weight × (1.0 - contrast × |sin(angle(t))|)
```

### SDF Generation

The signed distance from a point to the stroke boundary:

```
SDF(p) = min_over_all_strokes(|p - stroke(t)| - pen_width(t))
```

Positive outside, negative inside. GPU rendering uses a simple threshold.

## Memory Footprint

| Component | Size |
|-----------|------|
| MetaFontParams | 40 bytes |
| GlyphGenerator | 256 bytes |
| Single glyph SDF (32×32) | 1,024 bytes |
| SDF Atlas (512×512) | 256 KB |
| TextShaper | 512 bytes |

## License

MIT

For professional font authoring tools (ALICE-TypeForge), contact: sakamoro@ext.com

## Author

Moroya Sakamoto

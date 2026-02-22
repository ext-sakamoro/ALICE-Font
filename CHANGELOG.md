# Changelog

All notable changes to ALICE-Font will be documented in this file.

## [0.1.0] - 2026-02-23

### Added
- `MetaFontParams` — 40-byte parametric font descriptor with 6 presets
- `Stroke` / `PenModel` — variable-width pen along Bezier skeleton
- `GlyphGenerator` / `GlyphSdf` — procedural SDF glyph generation (A-Z, a-z, 0-9, punctuation)
- CJK stroke primitives (horizontal, vertical, dot, turning strokes)
- `SdfAtlas` — GPU-friendly texture atlas with LRU eviction
- `TextShaper` — kerning, horizontal advance, line layout
- `FontLicense` — 32-byte wire format for usage rights tracking
- `GameTextStyle` / `EffectStack` — SDF-based outline, shadow, glow, gradient effects
- `no_std` compatible (requires `alloc`)
- 98 unit tests covering all modules

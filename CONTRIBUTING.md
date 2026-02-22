# Contributing to ALICE-Font

## Build

```bash
cargo build
cargo build --features std
```

## Test

```bash
cargo test
```

## Lint

```bash
cargo clippy -- -W clippy::all
cargo fmt -- --check
cargo doc --no-deps 2>&1 | grep warning
```

## Design Constraints

- **no_std + alloc**: core rendering must compile without `std`. `alloc` is required for atlas and shaper.
- **40-byte params**: `MetaFontParams` is exactly 10 × f32 = 40 bytes. Do not add fields.
- **SDF-based**: all glyph output is signed distance fields, not bitmaps. Effects are distance-field operations.
- **Reciprocal multiplication**: avoid division in per-pixel loops. Pre-compute `1.0 / N` as constants.
- **Fixed-size SDF**: `GLYPH_SDF_SIZE` is a compile-time constant. Atlas tiles match this size.

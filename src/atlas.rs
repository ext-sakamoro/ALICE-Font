//! SDF Atlas — GPU-friendly glyph cache with LRU eviction
//!
//! Packs rendered GlyphSdf tiles into a power-of-two texture atlas.
//! Uses LRU eviction when the atlas is full.
//!
//! Atlas layout: NxN grid of GLYPH_SDF_SIZE×GLYPH_SDF_SIZE tiles
//! Total texture: (N × GLYPH_SDF_SIZE) × (N × GLYPH_SDF_SIZE) pixels
//!
//! License: MIT
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use crate::glyph::{GlyphGenerator, GlyphSdf, GLYPH_SDF_SIZE};
use crate::param::MetaFontParams;

/// Maximum atlas grid dimension (tiles per side)
pub const MAX_ATLAS_DIM: usize = 16;

/// Atlas entry — maps a character to its tile position
#[derive(Debug, Clone, Copy)]
pub struct AtlasEntry {
    /// Character stored
    pub codepoint: char,
    /// Tile column (0-based)
    pub tile_x: u16,
    /// Tile row (0-based)
    pub tile_y: u16,
    /// UV coordinates (normalized 0..1)
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_w: f32,
    pub uv_h: f32,
    /// Glyph metrics
    pub advance: f32,
    pub lsb: f32,
    /// LRU timestamp (higher = more recent)
    pub last_used: u32,
}

/// SDF texture atlas with LRU eviction
pub struct SdfAtlas {
    /// Atlas grid dimension (tiles per side)
    dim: usize,
    /// Pixel data (dim*GLYPH_SDF_SIZE × dim*GLYPH_SDF_SIZE)
    pixels: Vec<f32>,
    /// Entry table (one per tile slot)
    entries: Vec<Option<AtlasEntry>>,
    /// Current font parameters
    params: MetaFontParams,
    /// Glyph generator
    generator: GlyphGenerator,
    /// LRU clock
    clock: u32,
    /// Number of occupied tiles
    occupied: usize,
}

impl SdfAtlas {
    /// Create a new atlas with given grid dimension
    ///
    /// Total texture size: (dim × GLYPH_SDF_SIZE) × (dim × GLYPH_SDF_SIZE)
    pub fn new(dim: usize, params: MetaFontParams) -> Self {
        let dim = if dim > MAX_ATLAS_DIM { MAX_ATLAS_DIM } else { dim };
        let tex_size = dim * GLYPH_SDF_SIZE;
        Self {
            dim,
            pixels: vec![0.0f32; tex_size * tex_size],
            entries: vec![None; dim * dim],
            params,
            generator: GlyphGenerator::new(&params),
            clock: 0,
            occupied: 0,
        }
    }

    /// Total texture width/height in pixels
    pub fn texture_size(&self) -> usize {
        self.dim * GLYPH_SDF_SIZE
    }

    /// Number of tile slots
    pub fn capacity(&self) -> usize {
        self.dim * self.dim
    }

    /// Number of occupied tile slots
    pub fn occupied(&self) -> usize {
        self.occupied
    }

    /// Raw pixel data (row-major, f32 SDF values)
    pub fn pixels(&self) -> &[f32] {
        &self.pixels
    }

    /// Update font parameters (invalidates all cached glyphs)
    pub fn set_params(&mut self, params: MetaFontParams) {
        self.params = params;
        self.generator = GlyphGenerator::new(&params);
        self.clear();
    }

    /// Clear all cached glyphs
    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = None;
        }
        for px in self.pixels.iter_mut() {
            *px = 0.0;
        }
        self.occupied = 0;
        self.clock = 0;
    }

    /// Look up a character in the atlas
    pub fn lookup(&mut self, ch: char) -> Option<&AtlasEntry> {
        self.clock += 1;
        let clock = self.clock;
        for entry in self.entries.iter_mut() {
            if let Some(ref mut e) = entry {
                if e.codepoint == ch {
                    e.last_used = clock;
                    return Some(e);
                }
            }
        }
        None
    }

    /// Get or insert a glyph, returns atlas entry
    pub fn get_or_insert(&mut self, ch: char) -> AtlasEntry {
        self.clock += 1;
        let clock = self.clock;

        // Check if already cached
        for entry in self.entries.iter_mut() {
            if let Some(ref mut e) = entry {
                if e.codepoint == ch {
                    e.last_used = clock;
                    return *e;
                }
            }
        }

        // Generate glyph SDF (ASCII only)
        let ascii = if ch.is_ascii() { ch as u8 } else { b'?' };
        let sdf = self.generator.generate(ascii);

        // Find a free slot or evict LRU
        let slot = self.find_slot();

        // Compute tile position
        let tile_x = slot % self.dim;
        let tile_y = slot / self.dim;
        let tex_size = self.texture_size();
        let inv_tex = 1.0 / tex_size as f32;

        // Copy SDF data into atlas texture
        self.blit_tile(tile_x, tile_y, &sdf);

        let entry = AtlasEntry {
            codepoint: ch,
            tile_x: tile_x as u16,
            tile_y: tile_y as u16,
            uv_x: (tile_x * GLYPH_SDF_SIZE) as f32 * inv_tex,
            uv_y: (tile_y * GLYPH_SDF_SIZE) as f32 * inv_tex,
            uv_w: GLYPH_SDF_SIZE as f32 * inv_tex,
            uv_h: GLYPH_SDF_SIZE as f32 * inv_tex,
            advance: sdf.advance,
            lsb: sdf.lsb,
            last_used: clock,
        };

        if self.entries[slot].is_none() {
            self.occupied += 1;
        }
        self.entries[slot] = Some(entry);

        entry
    }

    /// Find a free slot or evict the least-recently-used entry
    fn find_slot(&self) -> usize {
        // First try to find an empty slot
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.is_none() {
                return i;
            }
        }

        // All slots full — find LRU
        let mut lru_idx = 0;
        let mut lru_time = u32::MAX;
        for (i, entry) in self.entries.iter().enumerate() {
            if let Some(ref e) = entry {
                if e.last_used < lru_time {
                    lru_time = e.last_used;
                    lru_idx = i;
                }
            }
        }
        lru_idx
    }

    /// Blit a glyph SDF into the atlas at the given tile position
    fn blit_tile(&mut self, tile_x: usize, tile_y: usize, sdf: &GlyphSdf) {
        let tex_w = self.texture_size();
        let base_x = tile_x * GLYPH_SDF_SIZE;
        let base_y = tile_y * GLYPH_SDF_SIZE;

        for row in 0..GLYPH_SDF_SIZE {
            for col in 0..GLYPH_SDF_SIZE {
                let src = row * GLYPH_SDF_SIZE + col;
                let dst = (base_y + row) * tex_w + (base_x + col);
                self.pixels[dst] = sdf.data[src];
            }
        }
    }

    /// Batch insert multiple characters
    pub fn preload(&mut self, chars: &[char]) {
        for &ch in chars {
            self.get_or_insert(ch);
        }
    }

    /// Get atlas entry without updating LRU (read-only peek)
    pub fn peek(&self, ch: char) -> Option<&AtlasEntry> {
        for entry in self.entries.iter() {
            if let Some(ref e) = entry {
                if e.codepoint == ch {
                    return Some(e);
                }
            }
        }
        None
    }

    /// Check if a character is cached
    pub fn contains(&self, ch: char) -> bool {
        self.peek(ch).is_some()
    }

    /// Get the pixel value at texture coordinates
    pub fn sample(&self, tex_x: usize, tex_y: usize) -> f32 {
        let tex_w = self.texture_size();
        if tex_x >= tex_w || tex_y >= tex_w {
            return 0.0;
        }
        self.pixels[tex_y * tex_w + tex_x]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_creation() {
        let atlas = SdfAtlas::new(4, MetaFontParams::sans_regular());
        assert_eq!(atlas.texture_size(), 4 * GLYPH_SDF_SIZE);
        assert_eq!(atlas.capacity(), 16);
        assert_eq!(atlas.occupied(), 0);
    }

    #[test]
    fn test_atlas_max_dim() {
        let atlas = SdfAtlas::new(999, MetaFontParams::sans_regular());
        assert_eq!(atlas.dim, MAX_ATLAS_DIM);
    }

    #[test]
    fn test_atlas_insert_and_lookup() {
        let mut atlas = SdfAtlas::new(4, MetaFontParams::sans_regular());
        let entry = atlas.get_or_insert('A');
        assert_eq!(entry.codepoint, 'A');
        assert!(entry.advance > 0.0);
        assert_eq!(atlas.occupied(), 1);

        // Lookup should find it
        let found = atlas.lookup('A');
        assert!(found.is_some());
        assert_eq!(found.unwrap().codepoint, 'A');
    }

    #[test]
    fn test_atlas_uv_coordinates() {
        let mut atlas = SdfAtlas::new(4, MetaFontParams::sans_regular());
        let entry = atlas.get_or_insert('A');
        // First tile should be at (0,0)
        assert!((entry.uv_x).abs() < 0.001);
        assert!((entry.uv_y).abs() < 0.001);
        assert!((entry.uv_w - 1.0 / 4.0).abs() < 0.001);
    }

    #[test]
    fn test_atlas_multiple_glyphs() {
        let mut atlas = SdfAtlas::new(4, MetaFontParams::sans_regular());
        atlas.preload(&['A', 'B', 'H', 'I']);
        assert_eq!(atlas.occupied(), 4);
        assert!(atlas.contains('A'));
        assert!(atlas.contains('B'));
        assert!(atlas.contains('H'));
        assert!(atlas.contains('I'));
        assert!(!atlas.contains('Z'));
    }

    #[test]
    fn test_atlas_duplicate_insert() {
        let mut atlas = SdfAtlas::new(4, MetaFontParams::sans_regular());
        atlas.get_or_insert('A');
        atlas.get_or_insert('A'); // duplicate
        assert_eq!(atlas.occupied(), 1); // should not double-count
    }

    #[test]
    fn test_atlas_lru_eviction() {
        // Create tiny atlas (2x2 = 4 slots)
        let mut atlas = SdfAtlas::new(2, MetaFontParams::sans_regular());
        assert_eq!(atlas.capacity(), 4);

        // Fill all 4 slots
        atlas.get_or_insert('A'); // LRU clock 2 (1 for failed lookup, 2 for insert)
        atlas.get_or_insert('B');
        atlas.get_or_insert('H');
        atlas.get_or_insert('I');
        assert_eq!(atlas.occupied(), 4);

        // Access 'B', 'H', 'I' to make 'A' the LRU
        atlas.lookup('B');
        atlas.lookup('H');
        atlas.lookup('I');

        // Insert 'T' — should evict 'A' (LRU)
        atlas.get_or_insert('T');
        assert_eq!(atlas.occupied(), 4);
        assert!(atlas.contains('T'));
        assert!(!atlas.contains('A')); // evicted
        assert!(atlas.contains('B'));
    }

    #[test]
    fn test_atlas_clear() {
        let mut atlas = SdfAtlas::new(4, MetaFontParams::sans_regular());
        atlas.preload(&['A', 'B', 'H']);
        assert_eq!(atlas.occupied(), 3);

        atlas.clear();
        assert_eq!(atlas.occupied(), 0);
        assert!(!atlas.contains('A'));
    }

    #[test]
    fn test_atlas_set_params() {
        let mut atlas = SdfAtlas::new(4, MetaFontParams::sans_regular());
        atlas.preload(&['A', 'B']);
        assert_eq!(atlas.occupied(), 2);

        // Changing params should clear cache
        atlas.set_params(MetaFontParams::sans_bold());
        assert_eq!(atlas.occupied(), 0);
    }

    #[test]
    fn test_atlas_sample() {
        let mut atlas = SdfAtlas::new(4, MetaFontParams::sans_regular());
        atlas.get_or_insert('A');

        // Sample within first tile — should have some non-zero values
        let mut has_nonzero = false;
        for y in 0..GLYPH_SDF_SIZE {
            for x in 0..GLYPH_SDF_SIZE {
                if atlas.sample(x, y).abs() > 0.001 {
                    has_nonzero = true;
                    break;
                }
            }
        }
        assert!(has_nonzero);
    }

    #[test]
    fn test_atlas_sample_out_of_bounds() {
        let atlas = SdfAtlas::new(4, MetaFontParams::sans_regular());
        assert!((atlas.sample(9999, 9999)).abs() < 0.001);
    }

    #[test]
    fn test_atlas_pixel_data_size() {
        let atlas = SdfAtlas::new(4, MetaFontParams::sans_regular());
        let tex = atlas.texture_size();
        assert_eq!(atlas.pixels().len(), tex * tex);
    }
}

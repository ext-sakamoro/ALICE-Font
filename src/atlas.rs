//! SDF Atlas — GPU-friendly glyph cache with LRU eviction
//!
//! Packs rendered `GlyphSdf` tiles into a power-of-two texture atlas.
//! Uses LRU eviction when the atlas is full.
//!
//! Atlas layout: `NxN` grid of `GLYPH_SDF_SIZE×GLYPH_SDF_SIZE` tiles
//! Total texture: (N × `GLYPH_SDF_SIZE`) × (N × `GLYPH_SDF_SIZE`) pixels
//!
//! License: MIT
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use crate::glyph::{dispatcher, GlyphGenerator, GlyphSdf, GLYPH_SDF_SIZE};
use crate::param::MetaFontParams;

/// Maximum atlas grid dimension (tiles per side) for the legacy single-page
/// `SdfAtlas`. New code should prefer [`SdfAtlasMulti`] which supports much
/// larger glyph populations (CJK).
pub const MAX_ATLAS_DIM: usize = 16;

/// Maximum tiles per side for a single page of [`SdfAtlasMulti`]. With the
/// default `GLYPH_SDF_SIZE = 32` this allows a 2048×2048 texture per page.
pub const MAX_ATLAS_DIM_PER_PAGE: usize = 64;

/// Maximum number of pages in [`SdfAtlasMulti`]. With the defaults above
/// this gives 8 × 4096 = 32,768 glyph slots, comfortably covering all
/// Joyo kanji plus kana plus ASCII.
pub const MAX_ATLAS_PAGES: usize = 8;

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
    /// Pixel data (dim*`GLYPH_SDF_SIZE` × dim*`GLYPH_SDF_SIZE`)
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
    /// Total texture size: (dim × `GLYPH_SDF_SIZE`) × (dim × `GLYPH_SDF_SIZE`)
    #[must_use]
    pub fn new(dim: usize, params: MetaFontParams) -> Self {
        let dim = if dim > MAX_ATLAS_DIM {
            MAX_ATLAS_DIM
        } else {
            dim
        };
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
    #[must_use]
    pub const fn texture_size(&self) -> usize {
        self.dim * GLYPH_SDF_SIZE
    }

    /// Number of tile slots
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.dim * self.dim
    }

    /// Number of occupied tile slots
    #[must_use]
    pub const fn occupied(&self) -> usize {
        self.occupied
    }

    /// Raw pixel data (row-major, f32 SDF values)
    #[must_use]
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
        for entry in &mut self.entries {
            *entry = None;
        }
        for px in &mut self.pixels {
            *px = 0.0;
        }
        self.occupied = 0;
        self.clock = 0;
    }

    /// Look up a character in the atlas
    pub fn lookup(&mut self, ch: char) -> Option<&AtlasEntry> {
        self.clock += 1;
        let clock = self.clock;
        for e in self.entries.iter_mut().flatten() {
            if e.codepoint == ch {
                e.last_used = clock;
                return Some(e);
            }
        }
        None
    }

    /// Get or insert a glyph, returns atlas entry
    pub fn get_or_insert(&mut self, ch: char) -> AtlasEntry {
        self.clock += 1;
        let clock = self.clock;

        // Check if already cached
        for e in self.entries.iter_mut().flatten() {
            if e.codepoint == ch {
                e.last_used = clock;
                return *e;
            }
        }

        // Generate glyph SDF via the Unicode-aware dispatcher. Non-ASCII
        // characters get a placeholder SDF until the corresponding script
        // module is implemented (see `docs/CJK_ROADMAP.md`).
        let sdf = dispatcher::generate(ch, &self.params);

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
    #[must_use]
    pub fn peek(&self, ch: char) -> Option<&AtlasEntry> {
        self.entries.iter().flatten().find(|e| e.codepoint == ch)
    }

    /// Check if a character is cached
    #[must_use]
    pub fn contains(&self, ch: char) -> bool {
        self.peek(ch).is_some()
    }

    /// Get the pixel value at texture coordinates
    #[must_use]
    pub fn sample(&self, tex_x: usize, tex_y: usize) -> f32 {
        let tex_w = self.texture_size();
        if tex_x >= tex_w || tex_y >= tex_w {
            return 0.0;
        }
        self.pixels[tex_y * tex_w + tex_x]
    }
}

// ============================================================================
// Multi-page atlas (for CJK / large glyph populations)
// ============================================================================

/// Atlas entry for [`SdfAtlasMulti`] — includes a page identifier so the
/// renderer can pick the right texture layer.
#[derive(Debug, Clone, Copy)]
pub struct AtlasEntryMulti {
    pub codepoint: char,
    pub page_id: u16,
    pub tile_x: u16,
    pub tile_y: u16,
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_w: f32,
    pub uv_h: f32,
    pub advance: f32,
    pub lsb: f32,
    pub last_used: u32,
}

/// A single page of [`SdfAtlasMulti`]. Each page is a square grid of tiles
/// of dimension `page_dim × page_dim`, each tile being `GLYPH_SDF_SIZE`
/// pixels on a side.
pub struct SdfAtlasPage {
    dim: usize,
    pixels: Vec<f32>,
    entries: Vec<Option<AtlasEntryMulti>>,
    occupied: usize,
}

impl SdfAtlasPage {
    fn new(dim: usize) -> Self {
        let tex_size = dim * GLYPH_SDF_SIZE;
        Self {
            dim,
            pixels: vec![0.0f32; tex_size * tex_size],
            entries: vec![None; dim * dim],
            occupied: 0,
        }
    }

    /// Total side length (in pixels) of this page's texture.
    #[must_use]
    pub const fn texture_size(&self) -> usize {
        self.dim * GLYPH_SDF_SIZE
    }

    /// Number of tile slots this page exposes.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.dim * self.dim
    }

    /// Number of tile slots currently in use.
    #[must_use]
    pub const fn occupied(&self) -> usize {
        self.occupied
    }

    /// Raw pixel data for GPU upload (row-major, f32 SDF values).
    #[must_use]
    pub fn pixels(&self) -> &[f32] {
        &self.pixels
    }
}

/// Multi-page SDF atlas — recommended for CJK / large glyph populations.
///
/// Looks up entries via a flat scan today; a future revision may switch to
/// a hash map once the `no_std` story is sorted out.
pub struct SdfAtlasMulti {
    pages: Vec<SdfAtlasPage>,
    page_dim: usize,
    params: MetaFontParams,
    clock: u32,
}

impl SdfAtlasMulti {
    /// Create a new multi-page atlas.
    ///
    /// `num_pages` is clamped to `[1, MAX_ATLAS_PAGES]` and `page_dim` to
    /// `[1, MAX_ATLAS_DIM_PER_PAGE]`.
    #[must_use]
    pub fn new(num_pages: usize, page_dim: usize, params: MetaFontParams) -> Self {
        let num_pages = num_pages.clamp(1, MAX_ATLAS_PAGES);
        let page_dim = page_dim.clamp(1, MAX_ATLAS_DIM_PER_PAGE);
        let mut pages = Vec::with_capacity(num_pages);
        for _ in 0..num_pages {
            pages.push(SdfAtlasPage::new(page_dim));
        }
        Self {
            pages,
            page_dim,
            params,
            clock: 0,
        }
    }

    /// Number of pages in this atlas.
    #[must_use]
    pub const fn num_pages(&self) -> usize {
        self.pages.len()
    }

    /// Per-page texture side length in pixels.
    #[must_use]
    pub const fn page_size(&self) -> usize {
        self.page_dim * GLYPH_SDF_SIZE
    }

    /// Total slot capacity across all pages.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.pages.len() * self.page_dim * self.page_dim
    }

    /// Total number of glyphs currently stored across all pages.
    #[must_use]
    pub fn occupied(&self) -> usize {
        self.pages.iter().map(|p| p.occupied).sum()
    }

    /// Access the raw pixels of a given page (for GPU upload).
    #[must_use]
    pub fn page_pixels(&self, page_id: usize) -> Option<&[f32]> {
        self.pages.get(page_id).map(SdfAtlasPage::pixels)
    }

    /// Look up a character without inserting; returns a copy if cached.
    #[must_use]
    pub fn peek(&self, ch: char) -> Option<AtlasEntryMulti> {
        for page in &self.pages {
            for entry in page.entries.iter().flatten() {
                if entry.codepoint == ch {
                    return Some(*entry);
                }
            }
        }
        None
    }

    /// Is `ch` currently cached?
    #[must_use]
    pub fn contains(&self, ch: char) -> bool {
        self.peek(ch).is_some()
    }

    /// Look up a character, refreshing its LRU timestamp if found.
    pub fn lookup(&mut self, ch: char) -> Option<AtlasEntryMulti> {
        self.clock += 1;
        let clock = self.clock;
        for page in &mut self.pages {
            for entry in page.entries.iter_mut().flatten() {
                if entry.codepoint == ch {
                    entry.last_used = clock;
                    return Some(*entry);
                }
            }
        }
        None
    }

    /// Get an existing entry or insert a new one, evicting an LRU entry if
    /// every page is full.
    pub fn get_or_insert(&mut self, ch: char) -> AtlasEntryMulti {
        self.clock += 1;
        let clock = self.clock;

        // Cached?
        for page in &mut self.pages {
            for entry in page.entries.iter_mut().flatten() {
                if entry.codepoint == ch {
                    entry.last_used = clock;
                    return *entry;
                }
            }
        }

        let sdf = dispatcher::generate(ch, &self.params);
        let (page_id, slot) = self.find_slot();
        let page = &mut self.pages[page_id];
        let tile_x = slot % page.dim;
        let tile_y = slot / page.dim;
        let tex_size = page.texture_size();
        let inv_tex = 1.0 / tex_size as f32;

        blit_tile(page, tile_x, tile_y, &sdf);

        let entry = AtlasEntryMulti {
            codepoint: ch,
            page_id: page_id as u16,
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
        if page.entries[slot].is_none() {
            page.occupied += 1;
        }
        page.entries[slot] = Some(entry);
        entry
    }

    /// Batch insert.
    pub fn preload(&mut self, chars: &[char]) {
        for &ch in chars {
            self.get_or_insert(ch);
        }
    }

    /// Empty every page.
    pub fn clear(&mut self) {
        for page in &mut self.pages {
            for entry in &mut page.entries {
                *entry = None;
            }
            for px in &mut page.pixels {
                *px = 0.0;
            }
            page.occupied = 0;
        }
        self.clock = 0;
    }

    /// Change the rendering parameters, dropping every cached glyph.
    pub fn set_params(&mut self, params: MetaFontParams) {
        self.params = params;
        self.clear();
    }

    /// Find the first empty slot across all pages, or evict the LRU
    /// entry if every page is full. Returns `(page_id, slot_index)`.
    fn find_slot(&self) -> (usize, usize) {
        for (pid, page) in self.pages.iter().enumerate() {
            for (i, entry) in page.entries.iter().enumerate() {
                if entry.is_none() {
                    return (pid, i);
                }
            }
        }
        // All pages full — find global LRU.
        let mut lru_page = 0;
        let mut lru_slot = 0;
        let mut lru_time = u32::MAX;
        for (pid, page) in self.pages.iter().enumerate() {
            for (i, entry) in page.entries.iter().enumerate() {
                if let Some(e) = entry {
                    if e.last_used < lru_time {
                        lru_time = e.last_used;
                        lru_page = pid;
                        lru_slot = i;
                    }
                }
            }
        }
        (lru_page, lru_slot)
    }
}

fn blit_tile(page: &mut SdfAtlasPage, tile_x: usize, tile_y: usize, sdf: &GlyphSdf) {
    let tex_w = page.texture_size();
    let base_x = tile_x * GLYPH_SDF_SIZE;
    let base_y = tile_y * GLYPH_SDF_SIZE;
    for row in 0..GLYPH_SDF_SIZE {
        for col in 0..GLYPH_SDF_SIZE {
            let src = row * GLYPH_SDF_SIZE + col;
            let dst = (base_y + row) * tex_w + (base_x + col);
            page.pixels[dst] = sdf.data[src];
        }
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

    // ====================================================================
    // SdfAtlasMulti
    // ====================================================================

    #[test]
    fn multi_atlas_creation() {
        let atlas = SdfAtlasMulti::new(3, 32, MetaFontParams::sans_regular());
        assert_eq!(atlas.num_pages(), 3);
        assert_eq!(atlas.page_size(), 32 * GLYPH_SDF_SIZE);
        assert_eq!(atlas.capacity(), 3 * 32 * 32);
        assert_eq!(atlas.occupied(), 0);
    }

    #[test]
    fn multi_atlas_clamps_dimensions() {
        let atlas = SdfAtlasMulti::new(999, 999, MetaFontParams::sans_regular());
        assert_eq!(atlas.num_pages(), MAX_ATLAS_PAGES);
        assert_eq!(atlas.page_size(), MAX_ATLAS_DIM_PER_PAGE * GLYPH_SDF_SIZE);
    }

    #[test]
    fn multi_atlas_clamps_to_one() {
        let atlas = SdfAtlasMulti::new(0, 0, MetaFontParams::sans_regular());
        assert_eq!(atlas.num_pages(), 1);
        assert_eq!(atlas.page_size(), GLYPH_SDF_SIZE);
    }

    #[test]
    fn multi_atlas_insert_ascii() {
        let mut atlas = SdfAtlasMulti::new(1, 4, MetaFontParams::sans_regular());
        let entry = atlas.get_or_insert('A');
        assert_eq!(entry.codepoint, 'A');
        assert_eq!(entry.page_id, 0);
        assert!(entry.advance > 0.0);
        assert_eq!(atlas.occupied(), 1);
    }

    #[test]
    fn multi_atlas_insert_hiragana_placeholder() {
        let mut atlas = SdfAtlasMulti::new(1, 4, MetaFontParams::sans_regular());
        let entry = atlas.get_or_insert('あ');
        assert_eq!(entry.codepoint, 'あ');
        assert!(entry.advance > 0.0);
        assert_eq!(atlas.occupied(), 1);
    }

    #[test]
    fn multi_atlas_pages_filled_in_order() {
        let mut atlas = SdfAtlasMulti::new(2, 2, MetaFontParams::sans_regular()); // 4 + 4 slots
                                                                                  // First 4 chars fill page 0, next 4 fill page 1.
        for c in 'A'..='D' {
            let e = atlas.get_or_insert(c);
            assert_eq!(e.page_id, 0);
        }
        for c in 'E'..='H' {
            let e = atlas.get_or_insert(c);
            assert_eq!(e.page_id, 1);
        }
        assert_eq!(atlas.occupied(), 8);
    }

    #[test]
    fn multi_atlas_lru_across_pages() {
        let mut atlas = SdfAtlasMulti::new(1, 2, MetaFontParams::sans_regular()); // 4 slots
        atlas.preload(&['A', 'B', 'H', 'I']);
        assert_eq!(atlas.occupied(), 4);

        // Touch every entry except 'A' so it becomes the global LRU.
        atlas.lookup('B');
        atlas.lookup('H');
        atlas.lookup('I');

        // Inserting 'T' should evict 'A'.
        atlas.get_or_insert('T');
        assert!(atlas.contains('T'));
        assert!(!atlas.contains('A'));
        assert!(atlas.contains('B'));
    }

    #[test]
    fn multi_atlas_duplicate_insert() {
        let mut atlas = SdfAtlasMulti::new(1, 4, MetaFontParams::sans_regular());
        atlas.get_or_insert('A');
        atlas.get_or_insert('A');
        assert_eq!(atlas.occupied(), 1);
    }

    #[test]
    fn multi_atlas_clear() {
        let mut atlas = SdfAtlasMulti::new(2, 2, MetaFontParams::sans_regular());
        atlas.preload(&['A', 'B', 'H']);
        assert_eq!(atlas.occupied(), 3);
        atlas.clear();
        assert_eq!(atlas.occupied(), 0);
        assert!(!atlas.contains('A'));
    }

    #[test]
    fn multi_atlas_set_params_invalidates() {
        let mut atlas = SdfAtlasMulti::new(1, 4, MetaFontParams::sans_regular());
        atlas.preload(&['A', 'B']);
        assert_eq!(atlas.occupied(), 2);
        atlas.set_params(MetaFontParams::sans_bold());
        assert_eq!(atlas.occupied(), 0);
    }

    #[test]
    fn multi_atlas_page_pixels_accessible() {
        let mut atlas = SdfAtlasMulti::new(2, 2, MetaFontParams::sans_regular());
        atlas.get_or_insert('A');
        let pixels = atlas.page_pixels(0).expect("page 0 should exist");
        let expected = atlas.page_size() * atlas.page_size();
        assert_eq!(pixels.len(), expected);
        // Some pixel should be non-zero since 'A' was rasterized.
        assert!(pixels.iter().any(|p| p.abs() > 0.001));
    }

    #[test]
    fn multi_atlas_page_pixels_out_of_range() {
        let atlas = SdfAtlasMulti::new(1, 2, MetaFontParams::sans_regular());
        assert!(atlas.page_pixels(5).is_none());
    }
}

//! Text Shaper — kerning, line layout, and text metrics
//!
//! Converts a string of characters into positioned glyphs
//! ready for rendering. Handles:
//! - Horizontal advance accumulation
//! - Kerning pair adjustments (hash-based O(1) lookup via sorted array + binary search)
//! - Line breaking
//! - Text metrics (width, height, baseline)
//!
//! License: MIT
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec::Vec;

use crate::param::MetaFontParams;
use crate::atlas::SdfAtlas;

/// Maximum kerning pairs in table
const MAX_KERN_PAIRS: usize = 64;

/// Positioned glyph for rendering
#[derive(Debug, Clone, Copy)]
pub struct ShapedGlyph {
    /// Character
    pub codepoint: char,
    /// X position (em units from line start)
    pub x: f32,
    /// Y position (em units from text origin)
    pub y: f32,
    /// Glyph advance width
    pub advance: f32,
    /// Left side bearing
    pub lsb: f32,
}

/// Line of shaped text
#[derive(Debug, Clone)]
pub struct ShapedLine {
    /// Glyphs in this line
    pub glyphs: Vec<ShapedGlyph>,
    /// Total line width (em units)
    pub width: f32,
    /// Line Y offset
    pub y_offset: f32,
}

/// Kerning pair — stored as a packed u64 key for O(log n) binary search.
///
/// Key encoding: `(left as u32) << 32 | (right as u32)`
/// This lets us sort and binary-search on a single integer comparison
/// instead of two char comparisons, and keeps each entry to 12 bytes
/// (vs. two chars + f32 = 12 bytes, same size but simpler comparison).
#[derive(Debug, Clone, Copy)]
struct KernEntry {
    /// Packed key: high 32 bits = left char, low 32 bits = right char
    key: u64,
    /// Kerning adjustment (em units, typically negative)
    adjustment: f32,
}

impl KernEntry {
    #[inline(always)]
    fn make_key(left: char, right: char) -> u64 {
        ((left as u64) << 32) | (right as u64)
    }

    fn new(left: char, right: char, adjustment: f32) -> Self {
        Self { key: Self::make_key(left, right), adjustment }
    }
}

/// Text shaper with kerning and layout
pub struct TextShaper {
    /// Font parameters
    params: MetaFontParams,
    /// Kerning table — sorted by key for O(log n) binary search.
    /// Invariant: always sorted ascending by `KernEntry::key`.
    kern_table: Vec<KernEntry>,
    /// Line height multiplier
    line_height: f32,
    /// Letter spacing (additional, em units)
    letter_spacing: f32,
    /// Word spacing multiplier (relative to space advance)
    word_spacing: f32,
}

impl TextShaper {
    /// Create shaper with default settings
    pub fn new(params: MetaFontParams) -> Self {
        let mut shaper = Self {
            params,
            kern_table: Vec::new(),
            line_height: 1.2,
            letter_spacing: 0.0,
            word_spacing: 1.0,
        };
        shaper.build_default_kern_table();
        shaper
    }

    /// Set line height multiplier (default 1.2)
    pub fn set_line_height(&mut self, h: f32) {
        self.line_height = h;
    }

    /// Set additional letter spacing (em units)
    pub fn set_letter_spacing(&mut self, s: f32) {
        self.letter_spacing = s;
    }

    /// Set word spacing multiplier (default 1.0)
    pub fn set_word_spacing(&mut self, s: f32) {
        self.word_spacing = s;
    }

    /// Build default kerning table for common Latin pairs.
    ///
    /// After inserting all pairs the table is sorted so that `kern()`
    /// can use `binary_search_by_key` — O(log n) instead of O(n).
    fn build_default_kern_table(&mut self) {
        let kern_data: &[(char, char, f32)] = &[
            // Diagonal pairs
            ('A', 'V', -0.04),
            ('A', 'W', -0.03),
            ('A', 'Y', -0.04),
            ('A', 'T', -0.04),
            ('V', 'A', -0.04),
            ('W', 'A', -0.03),
            ('Y', 'A', -0.04),
            ('T', 'A', -0.04),
            // Round + straight
            ('T', 'o', -0.03),
            ('T', 'a', -0.03),
            ('T', 'e', -0.03),
            ('L', 'T', -0.03),
            ('L', 'V', -0.03),
            ('L', 'Y', -0.03),
            // Lowercase
            ('r', 'a', -0.01),
            ('r', 'o', -0.01),
            ('f', 'a', -0.01),
            ('f', 'o', -0.01),
        ];

        self.kern_table.clear();
        for &(l, r, adj) in kern_data {
            if self.kern_table.len() >= MAX_KERN_PAIRS {
                break;
            }
            self.kern_table.push(KernEntry::new(l, r, adj));
        }
        // Sort once so binary_search_by_key works in kern()
        self.kern_table.sort_unstable_by_key(|e| e.key);
    }

    /// Add a custom kerning pair.
    ///
    /// Updates the existing entry if the pair is already present,
    /// otherwise inserts at the correct sorted position to keep
    /// the table sorted for O(log n) lookup.
    pub fn add_kern_pair(&mut self, left: char, right: char, adjustment: f32) {
        let key = KernEntry::make_key(left, right);
        match self.kern_table.binary_search_by_key(&key, |e| e.key) {
            Ok(idx) => {
                // Update existing pair in place
                self.kern_table[idx].adjustment = adjustment;
            }
            Err(idx) => {
                // Insert at sorted position (keeps invariant without a full re-sort)
                if self.kern_table.len() < MAX_KERN_PAIRS {
                    self.kern_table.insert(idx, KernEntry { key, adjustment });
                }
                // If the table is full, silently drop (same behaviour as before)
            }
        }
    }

    /// Look up kerning adjustment for a character pair — O(log n) binary search.
    pub fn kern(&self, left: char, right: char) -> f32 {
        let key = KernEntry::make_key(left, right);
        match self.kern_table.binary_search_by_key(&key, |e| e.key) {
            Ok(idx) => self.kern_table[idx].adjustment,
            Err(_) => 0.0,
        }
    }

    /// Shape a single line of text using atlas for metrics
    pub fn shape_line(&self, text: &str, atlas: &mut SdfAtlas) -> ShapedLine {
        let mut glyphs = Vec::new();
        let mut cursor_x: f32 = 0.0;
        let mut prev_char: Option<char> = None;

        let space_advance = self.params.width * 0.3;

        for ch in text.chars() {
            if ch == ' ' {
                cursor_x += space_advance * self.word_spacing + self.letter_spacing;
                prev_char = Some(ch);
                continue;
            }

            // Apply kerning
            if let Some(prev) = prev_char {
                cursor_x += self.kern(prev, ch);
            }

            // Get glyph metrics from atlas
            let entry = atlas.get_or_insert(ch);

            glyphs.push(ShapedGlyph {
                codepoint: ch,
                x: cursor_x + entry.lsb,
                y: 0.0,
                advance: entry.advance,
                lsb: entry.lsb,
            });

            cursor_x += entry.advance + self.letter_spacing;
            prev_char = Some(ch);
        }

        ShapedLine {
            width: cursor_x,
            glyphs,
            y_offset: 0.0,
        }
    }

    /// Shape text with automatic line breaking at max_width
    pub fn shape_text(&self, text: &str, atlas: &mut SdfAtlas, max_width: f32) -> Vec<ShapedLine> {
        let mut lines: Vec<ShapedLine> = Vec::new();
        let line_step = self.line_height * (self.params.ascender + self.params.descender);

        // Split by explicit newlines first
        for raw_line in text.split('\n') {
            if max_width <= 0.0 || max_width >= 1e6 {
                // No wrapping
                let mut shaped = self.shape_line(raw_line, atlas);
                shaped.y_offset = lines.len() as f32 * line_step;
                lines.push(shaped);
            } else {
                // Word-wrap
                let words: Vec<&str> = raw_line.split(' ').collect();
                let mut current_line = alloc::string::String::new();

                for (i, word) in words.iter().enumerate() {
                    let test_line = if current_line.is_empty() {
                        alloc::string::String::from(*word)
                    } else {
                        let mut s = current_line.clone();
                        s.push(' ');
                        s.push_str(word);
                        s
                    };

                    let test_shaped = self.shape_line(&test_line, atlas);

                    if test_shaped.width > max_width && !current_line.is_empty() {
                        // Emit current line
                        let mut shaped = self.shape_line(&current_line, atlas);
                        shaped.y_offset = lines.len() as f32 * line_step;
                        lines.push(shaped);
                        current_line = alloc::string::String::from(*word);
                    } else {
                        current_line = test_line;
                    }

                    // Last word in line
                    if i == words.len() - 1 && !current_line.is_empty() {
                        let mut shaped = self.shape_line(&current_line, atlas);
                        shaped.y_offset = lines.len() as f32 * line_step;
                        lines.push(shaped);
                    }
                }

                // Handle empty lines
                if words.is_empty() {
                    lines.push(ShapedLine {
                        glyphs: Vec::new(),
                        width: 0.0,
                        y_offset: lines.len() as f32 * line_step,
                    });
                }
            }
        }

        lines
    }

    /// Measure text width without full shaping
    pub fn measure_width(&self, text: &str, atlas: &mut SdfAtlas) -> f32 {
        self.shape_line(text, atlas).width
    }

    /// Compute text bounding box (width, height) for multi-line text
    pub fn measure_text(&self, text: &str, atlas: &mut SdfAtlas, max_width: f32) -> (f32, f32) {
        let lines = self.shape_text(text, atlas, max_width);
        let mut total_width: f32 = 0.0;
        for line in &lines {
            if line.width > total_width {
                total_width = line.width;
            }
        }
        let line_step = self.line_height * (self.params.ascender + self.params.descender);
        let total_height = lines.len() as f32 * line_step;
        (total_width, total_height)
    }
}

/// Text metrics for a shaped result
#[derive(Debug, Clone, Copy)]
pub struct TextMetrics {
    /// Total width (em units)
    pub width: f32,
    /// Total height (em units)
    pub height: f32,
    /// Number of lines
    pub line_count: usize,
    /// Number of glyphs
    pub glyph_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_atlas() -> SdfAtlas {
        SdfAtlas::new(4, MetaFontParams::sans_regular())
    }

    #[test]
    fn test_shaper_creation() {
        let shaper = TextShaper::new(MetaFontParams::sans_regular());
        assert!(shaper.line_height > 0.0);
    }

    #[test]
    fn test_kern_lookup() {
        let shaper = TextShaper::new(MetaFontParams::sans_regular());
        let k = shaper.kern('A', 'V');
        assert!(k < 0.0); // A-V should kern negative (tighter)
        let k_none = shaper.kern('H', 'I');
        assert!((k_none).abs() < 0.001); // No kerning for H-I
    }

    #[test]
    fn test_custom_kern_pair() {
        let mut shaper = TextShaper::new(MetaFontParams::sans_regular());
        shaper.add_kern_pair('X', 'Y', -0.05);
        assert!((shaper.kern('X', 'Y') - (-0.05)).abs() < 0.001);
    }

    /// Verify that add_kern_pair updates an existing entry correctly.
    #[test]
    fn test_update_existing_kern_pair() {
        let mut shaper = TextShaper::new(MetaFontParams::sans_regular());
        // 'A','V' is in the default table with -0.04
        shaper.add_kern_pair('A', 'V', -0.10);
        assert!((shaper.kern('A', 'V') - (-0.10)).abs() < 0.001);
    }

    /// Kern table must stay sorted so binary search works after custom inserts.
    #[test]
    fn test_kern_table_sorted_after_insert() {
        let mut shaper = TextShaper::new(MetaFontParams::sans_regular());
        shaper.add_kern_pair('Z', 'Z', -0.02);
        shaper.add_kern_pair('A', 'A', -0.01);
        // Table must be sorted ascending by key
        for w in shaper.kern_table.windows(2) {
            assert!(w[0].key <= w[1].key, "kern_table not sorted");
        }
    }

    #[test]
    fn test_shape_single_char() {
        let shaper = TextShaper::new(MetaFontParams::sans_regular());
        let mut atlas = make_atlas();
        let line = shaper.shape_line("A", &mut atlas);
        assert_eq!(line.glyphs.len(), 1);
        assert!(line.width > 0.0);
        assert_eq!(line.glyphs[0].codepoint, 'A');
    }

    #[test]
    fn test_shape_word() {
        let shaper = TextShaper::new(MetaFontParams::sans_regular());
        let mut atlas = make_atlas();
        let line = shaper.shape_line("HAIL", &mut atlas);
        assert_eq!(line.glyphs.len(), 4);
        // Each glyph should advance to the right
        for i in 1..line.glyphs.len() {
            assert!(line.glyphs[i].x > line.glyphs[i - 1].x);
        }
    }

    #[test]
    fn test_shape_with_space() {
        let shaper = TextShaper::new(MetaFontParams::sans_regular());
        let mut atlas = make_atlas();
        let line = shaper.shape_line("A B", &mut atlas);
        assert_eq!(line.glyphs.len(), 2); // Spaces aren't glyphs
        // B should be further right than A's advance
        assert!(line.glyphs[1].x > line.glyphs[0].advance);
    }

    #[test]
    fn test_shape_multiline() {
        let shaper = TextShaper::new(MetaFontParams::sans_regular());
        let mut atlas = make_atlas();
        let lines = shaper.shape_text("AB\nHI", &mut atlas, 0.0);
        assert_eq!(lines.len(), 2);
        assert!(lines[1].y_offset > lines[0].y_offset);
    }

    #[test]
    fn test_measure_width() {
        let shaper = TextShaper::new(MetaFontParams::sans_regular());
        let mut atlas = make_atlas();
        let w1 = shaper.measure_width("A", &mut atlas);
        let w2 = shaper.measure_width("AA", &mut atlas);
        assert!(w2 > w1);
    }

    #[test]
    fn test_letter_spacing() {
        let mut shaper = TextShaper::new(MetaFontParams::sans_regular());
        let mut atlas = make_atlas();
        let w_normal = shaper.measure_width("AB", &mut atlas);

        shaper.set_letter_spacing(0.1);
        let w_spaced = shaper.measure_width("AB", &mut atlas);
        assert!(w_spaced > w_normal);
    }

    #[test]
    fn test_word_wrap() {
        let shaper = TextShaper::new(MetaFontParams::sans_regular());
        let mut atlas = make_atlas();

        // Very narrow max width should force wrapping
        let lines = shaper.shape_text("AB HI", &mut atlas, 0.3);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_measure_text() {
        let shaper = TextShaper::new(MetaFontParams::sans_regular());
        let mut atlas = make_atlas();
        let (w, h) = shaper.measure_text("AB\nHI", &mut atlas, 0.0);
        assert!(w > 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn test_empty_text() {
        let shaper = TextShaper::new(MetaFontParams::sans_regular());
        let mut atlas = make_atlas();
        let line = shaper.shape_line("", &mut atlas);
        assert_eq!(line.glyphs.len(), 0);
        assert!((line.width).abs() < 0.001);
    }

    #[test]
    fn test_shaper_serif_vs_sans() {
        let mut atlas_sans = SdfAtlas::new(4, MetaFontParams::sans_regular());
        let mut atlas_serif = SdfAtlas::new(4, MetaFontParams::serif_regular());
        let shaper_sans = TextShaper::new(MetaFontParams::sans_regular());
        let shaper_serif = TextShaper::new(MetaFontParams::serif_regular());

        let w_sans = shaper_sans.measure_width("A", &mut atlas_sans);
        let w_serif = shaper_serif.measure_width("A", &mut atlas_serif);
        // Both should produce positive width
        assert!(w_sans > 0.0);
        assert!(w_serif > 0.0);
    }
}

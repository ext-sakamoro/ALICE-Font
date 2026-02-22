//! Game text primitives — SDF-based text effects for game UI
//!
//! Applies visual effects (outline, shadow, glow, gradient) to SDF
//! glyphs using distance-field evaluation. Each effect is computed
//! per-pixel from the signed distance value, then composited via
//! Porter-Duff "over" blending.
//!
//! License: MIT
//! Author: Moroya Sakamoto

use crate::glyph::{GlyphSdf, GLYPH_SDF_SIZE};

/// RGBA color (16 bytes, #[repr(C)])
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4 {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Linear interpolation
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// Premultiply alpha
    #[inline]
    pub fn premultiply(self) -> Self {
        Self {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }

    /// Porter-Duff "over" compositing (self over dst)
    #[inline]
    pub fn over(self, dst: Self) -> Self {
        let sa = self.a;
        let da = dst.a * (1.0 - sa);
        let out_a = sa + da;
        if out_a < 1e-6 {
            return Self::TRANSPARENT;
        }
        let inv_a = 1.0 / out_a;
        Self {
            r: (self.r * sa + dst.r * da) * inv_a,
            g: (self.g * sa + dst.g * da) * inv_a,
            b: (self.b * sa + dst.b * da) * inv_a,
            a: out_a,
        }
    }
}

/// SDF-based text effect
#[derive(Debug, Clone, Copy)]
pub enum TextEffect {
    /// Outline with specified color and width
    Outline { color: Color4, width: f32 },
    /// Drop shadow with offset and softness
    Shadow {
        color: Color4,
        offset_x: f32,
        offset_y: f32,
        softness: f32,
    },
    /// Outer glow with radius and falloff exponent
    Glow {
        color: Color4,
        radius: f32,
        falloff: f32,
    },
    /// Inner shadow
    InnerShadow {
        color: Color4,
        offset_x: f32,
        offset_y: f32,
        softness: f32,
    },
    /// Vertical gradient (top-to-bottom with optional radial mixing)
    Gradient {
        color_top: Color4,
        color_bottom: Color4,
        radial_mix: f32,
    },
}

/// Maximum effects in a stack
pub const MAX_EFFECTS: usize = 8;

/// Fixed-capacity effect stack
#[derive(Debug, Clone)]
pub struct EffectStack {
    effects: [Option<TextEffect>; MAX_EFFECTS],
    count: usize,
}

impl EffectStack {
    pub fn new() -> Self {
        Self {
            effects: [None; MAX_EFFECTS],
            count: 0,
        }
    }

    pub fn push(&mut self, effect: TextEffect) -> bool {
        if self.count < MAX_EFFECTS {
            self.effects[self.count] = Some(effect);
            self.count += 1;
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn get(&self, index: usize) -> Option<&TextEffect> {
        if index < self.count {
            self.effects[index].as_ref()
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.effects = [None; MAX_EFFECTS];
    }
}

impl Default for EffectStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete text style definition for game rendering
#[derive(Debug, Clone)]
pub struct GameTextStyle {
    /// Base fill color
    pub base_color: Color4,
    /// SDF boundary threshold (default 0.0)
    pub threshold: f32,
    /// Anti-aliasing width (default 0.03)
    pub aa_width: f32,
    /// Effect stack
    pub effects: EffectStack,
}

impl GameTextStyle {
    /// Default style (white text, no effects)
    pub fn default_style() -> Self {
        Self {
            base_color: Color4::WHITE,
            threshold: 0.0,
            aa_width: 0.03,
            effects: EffectStack::new(),
        }
    }

    /// White text with black outline
    pub fn outlined() -> Self {
        let mut effects = EffectStack::new();
        effects.push(TextEffect::Outline {
            color: Color4::BLACK,
            width: 0.05,
        });
        Self {
            base_color: Color4::WHITE,
            threshold: 0.0,
            aa_width: 0.03,
            effects,
        }
    }

    /// White text with drop shadow
    pub fn shadowed() -> Self {
        let mut effects = EffectStack::new();
        effects.push(TextEffect::Shadow {
            color: Color4::new(0.0, 0.0, 0.0, 0.6),
            offset_x: 0.03,
            offset_y: -0.03,
            softness: 0.04,
        });
        Self {
            base_color: Color4::WHITE,
            threshold: 0.0,
            aa_width: 0.03,
            effects,
        }
    }

    /// Neon glow effect
    pub fn neon() -> Self {
        let mut effects = EffectStack::new();
        effects.push(TextEffect::Glow {
            color: Color4::new(0.0, 1.0, 0.5, 0.8),
            radius: 0.15,
            falloff: 2.0,
        });
        Self {
            base_color: Color4::new(0.0, 1.0, 0.5, 1.0),
            threshold: 0.0,
            aa_width: 0.02,
            effects,
        }
    }
}

impl Default for GameTextStyle {
    fn default() -> Self {
        Self::default_style()
    }
}

/// SDF glyph tile size for styled output
pub const STYLED_GLYPH_SIZE: usize = GLYPH_SDF_SIZE;
const STYLED_PIXEL_COUNT: usize = STYLED_GLYPH_SIZE * STYLED_GLYPH_SIZE;

/// FNV-1a hash (file-local)
#[inline(always)]
fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Styled glyph output (32x32 RGBA)
pub struct StyledGlyph {
    pub pixels: [Color4; STYLED_PIXEL_COUNT],
    pub content_hash: u64,
    pub advance: f32,
}

/// Branchless smoothstep: smooth Hermite interpolation on [edge0, edge1]
#[inline(always)]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let range = edge1 - edge0;
    if range.abs() < 1e-10 {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    let t = (x - edge0) / range;
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Apply a single effect to a pixel given SDF distance and UV coordinates
fn evaluate_effect(effect: &TextEffect, d: f32, u: f32, v: f32, sdf: &GlyphSdf, aa: f32) -> Color4 {
    match *effect {
        TextEffect::Outline { color, width } => {
            // Ring between -width and 0 on the SDF
            let outer = smoothstep(-width - aa, -width, d);
            let inner = smoothstep(-aa, 0.0, d);
            let alpha = outer * (1.0 - inner);
            Color4::new(color.r, color.g, color.b, color.a * alpha)
        }
        TextEffect::Shadow {
            color,
            offset_x,
            offset_y,
            softness,
        } => {
            // Sample SDF at offset position
            let su = u - offset_x;
            let sv = v - offset_y;
            let sd = if (0.0..=1.0).contains(&su) && (0.0..=1.0).contains(&sv) {
                sdf.sample(su, sv)
            } else {
                1.0 // outside
            };
            let alpha = smoothstep(softness, -softness, sd);
            Color4::new(color.r, color.g, color.b, color.a * alpha)
        }
        TextEffect::Glow {
            color,
            radius,
            falloff,
        } => {
            if d <= 0.0 {
                // Inside glyph — no glow contribution (base color handles it)
                Color4::TRANSPARENT
            } else {
                let t = d / radius;
                let t = if t > 1.0 { 1.0 } else { t };
                // Exponential falloff
                let alpha = 1.0 - pow_approx(t, falloff);
                let alpha = if alpha < 0.0 { 0.0 } else { alpha };
                Color4::new(color.r, color.g, color.b, color.a * alpha)
            }
        }
        TextEffect::InnerShadow {
            color,
            offset_x,
            offset_y,
            softness,
        } => {
            if d >= 0.0 {
                // Outside glyph — no inner shadow
                Color4::TRANSPARENT
            } else {
                let su = u - offset_x;
                let sv = v - offset_y;
                let sd = if (0.0..=1.0).contains(&su) && (0.0..=1.0).contains(&sv) {
                    sdf.sample(su, sv)
                } else {
                    1.0
                };
                // Inner shadow: darken where the offset SDF is near the edge
                let alpha = smoothstep(-softness, softness, sd);
                Color4::new(color.r, color.g, color.b, color.a * alpha)
            }
        }
        TextEffect::Gradient {
            color_top,
            color_bottom,
            radial_mix,
        } => {
            if d >= 0.0 {
                Color4::TRANSPARENT
            } else {
                // v=0 is bottom, v=1 is top
                let linear_t = v;
                // Radial component: 0 at center, 1 at edge
                let cu = u - 0.5;
                let cv = v - 0.5;
                let radial_t = (cu * cu + cv * cv).min(0.25) * 4.0;
                let t = linear_t * (1.0 - radial_mix) + radial_t * radial_mix;
                let t = t.clamp(0.0, 1.0);
                color_bottom.lerp(color_top, t)
            }
        }
    }
}

/// Approximate power function for positive base and exponent
#[inline(always)]
fn pow_approx(base: f32, exp: f32) -> f32 {
    if base <= 0.0 {
        return 0.0;
    }
    if (exp - 1.0).abs() < 1e-6 {
        return base;
    }
    if (exp - 2.0).abs() < 1e-6 {
        return base * base;
    }
    // exp2(exp * log2(base)) approximation via bit manipulation
    let log2_base = {
        let bits = f32::to_bits(base);
        let e = ((bits >> 23) & 0xFF) as f32 - 127.0;
        let m = f32::from_bits((bits & 0x007FFFFF) | 0x3F800000) - 1.0;
        e + m * (1.0 + m * (-0.34484 + m * 0.09556))
    };
    let val = exp * log2_base;
    // exp2 approximation
    let floor_val = val as i32;
    let frac = val - floor_val as f32;
    let exp2_frac =
        1.0 + frac * (core::f32::consts::LN_2 + frac * (0.240_226_5 + frac * 0.055_499_3));
    let bits = ((floor_val + 127) as u32) << 23;
    f32::from_bits(bits) * exp2_frac
}

/// Apply style to a single SDF glyph, producing a 32x32 RGBA tile
pub fn style_glyph(sdf: &GlyphSdf, style: &GameTextStyle) -> StyledGlyph {
    let mut pixels = [Color4::TRANSPARENT; STYLED_PIXEL_COUNT];
    let size = STYLED_GLYPH_SIZE;
    let inv_size = 1.0 / (size - 1) as f32;

    for py in 0..size {
        for px in 0..size {
            let u = px as f32 * inv_size;
            let v = py as f32 * inv_size;
            let d = sdf.sample(u, v) - style.threshold;

            // Start with transparent background
            let mut color = Color4::TRANSPARENT;

            // Apply effects from bottom to top (first effect = bottom layer)
            for i in 0..style.effects.len() {
                if let Some(effect) = style.effects.get(i) {
                    let effect_color = evaluate_effect(effect, d, u, v, sdf, style.aa_width);
                    if effect_color.a > 1e-6 {
                        color = effect_color.over(color);
                    }
                }
            }

            // Apply base fill (inside the glyph)
            let base_alpha = smoothstep(style.aa_width, -style.aa_width, d);
            if base_alpha > 1e-6 {
                let base = Color4::new(
                    style.base_color.r,
                    style.base_color.g,
                    style.base_color.b,
                    style.base_color.a * base_alpha,
                );
                color = base.over(color);
            }

            pixels[py * size + px] = color;
        }
    }

    // Compute content hash from pixel data
    let mut hash_buf = [0u8; 16];
    // Sample a few representative pixels for the hash
    let p0 = &pixels[0];
    let p_mid = &pixels[STYLED_PIXEL_COUNT / 2];
    hash_buf[0..4].copy_from_slice(&p0.r.to_le_bytes());
    hash_buf[4..8].copy_from_slice(&p0.a.to_le_bytes());
    hash_buf[8..12].copy_from_slice(&p_mid.r.to_le_bytes());
    hash_buf[12..16].copy_from_slice(&p_mid.a.to_le_bytes());

    StyledGlyph {
        pixels,
        content_hash: fnv1a(&hash_buf),
        advance: sdf.advance,
    }
}

/// Batch-style multiple glyphs with the same style
#[cfg(feature = "std")]
pub fn style_glyphs_batch(
    sdfs: &[&GlyphSdf],
    style: &GameTextStyle,
) -> alloc::vec::Vec<StyledGlyph> {
    sdfs.iter().map(|sdf| style_glyph(sdf, style)).collect()
}

#[cfg(not(feature = "std"))]
pub fn style_glyphs_batch(
    sdfs: &[&GlyphSdf],
    style: &GameTextStyle,
) -> alloc::vec::Vec<StyledGlyph> {
    sdfs.iter().map(|sdf| style_glyph(sdf, style)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glyph::GlyphGenerator;
    use crate::param::MetaFontParams;

    #[test]
    fn test_color4_size() {
        assert_eq!(core::mem::size_of::<Color4>(), 16);
    }

    #[test]
    fn test_color4_lerp() {
        let a = Color4::BLACK;
        let b = Color4::WHITE;
        let mid = a.lerp(b, 0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color4_premultiply() {
        let c = Color4::new(1.0, 0.5, 0.0, 0.5);
        let pm = c.premultiply();
        assert!((pm.r - 0.5).abs() < 0.01);
        assert!((pm.g - 0.25).abs() < 0.01);
        assert!((pm.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color4_over_compositing() {
        // Opaque over anything = opaque
        let result = Color4::WHITE.over(Color4::BLACK);
        assert!((result.r - 1.0).abs() < 0.01);
        // Transparent over opaque = opaque
        let result = Color4::TRANSPARENT.over(Color4::BLACK);
        assert!((result.r - 0.0).abs() < 0.01);
        assert!((result.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_effect_stack() {
        let mut stack = EffectStack::new();
        assert!(stack.is_empty());
        stack.push(TextEffect::Outline {
            color: Color4::BLACK,
            width: 0.05,
        });
        assert_eq!(stack.len(), 1);
        assert!(stack.get(0).is_some());
        assert!(stack.get(1).is_none());
    }

    #[test]
    fn test_effect_stack_max_capacity() {
        let mut stack = EffectStack::new();
        for _ in 0..MAX_EFFECTS {
            assert!(stack.push(TextEffect::Outline {
                color: Color4::BLACK,
                width: 0.05
            }));
        }
        assert!(!stack.push(TextEffect::Outline {
            color: Color4::BLACK,
            width: 0.05
        }));
        assert_eq!(stack.len(), MAX_EFFECTS);
    }

    #[test]
    fn test_smoothstep_boundaries() {
        assert!((smoothstep(0.0, 1.0, -0.1) - 0.0).abs() < 0.01);
        assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 0.01);
        assert!((smoothstep(0.0, 1.0, 1.1) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_style_glyph_default() {
        let params = MetaFontParams::sans_bold();
        let gen = GlyphGenerator::new(&params);
        let sdf = gen.generate(b'A');
        let style = GameTextStyle::default_style();
        let styled = style_glyph(&sdf, &style);
        assert!(styled.advance > 0.0);
        // Should have some non-transparent pixels
        let mut has_color = false;
        for p in &styled.pixels {
            if p.a > 0.1 {
                has_color = true;
                break;
            }
        }
        assert!(has_color, "Styled glyph should have visible pixels");
    }

    #[test]
    fn test_style_glyph_outlined() {
        let params = MetaFontParams::sans_bold();
        let gen = GlyphGenerator::new(&params);
        let sdf = gen.generate(b'O');
        let style = GameTextStyle::outlined();
        let styled = style_glyph(&sdf, &style);
        assert!(styled.advance > 0.0);
    }

    #[test]
    fn test_style_glyph_neon() {
        let params = MetaFontParams::sans_regular();
        let gen = GlyphGenerator::new(&params);
        let sdf = gen.generate(b'A');
        let style = GameTextStyle::neon();
        let styled = style_glyph(&sdf, &style);
        // Neon glow should produce pixels outside the glyph boundary
        let mut has_glow = false;
        for py in 0..STYLED_GLYPH_SIZE {
            for px in 0..STYLED_GLYPH_SIZE {
                let u = px as f32 / (STYLED_GLYPH_SIZE - 1) as f32;
                let v = py as f32 / (STYLED_GLYPH_SIZE - 1) as f32;
                let d = sdf.sample(u, v);
                if d > 0.0 && styled.pixels[py * STYLED_GLYPH_SIZE + px].a > 0.01 {
                    has_glow = true;
                    break;
                }
            }
            if has_glow {
                break;
            }
        }
        assert!(has_glow, "Neon style should produce glow outside glyph");
    }

    #[test]
    fn test_hash_determinism() {
        let params = MetaFontParams::sans_bold();
        let gen = GlyphGenerator::new(&params);
        let sdf = gen.generate(b'A');
        let style = GameTextStyle::default_style();
        let s1 = style_glyph(&sdf, &style);
        let s2 = style_glyph(&sdf, &style);
        assert_eq!(s1.content_hash, s2.content_hash);
        assert_ne!(s1.content_hash, 0);
    }

    #[test]
    fn test_presets_create() {
        let _ = GameTextStyle::default_style();
        let _ = GameTextStyle::outlined();
        let _ = GameTextStyle::shadowed();
        let _ = GameTextStyle::neon();
    }

    #[test]
    fn test_batch_style() {
        let params = MetaFontParams::sans_bold();
        let gen = GlyphGenerator::new(&params);
        let sdf_a = gen.generate(b'A');
        let sdf_b = gen.generate(b'B');
        let style = GameTextStyle::default_style();
        let result = style_glyphs_batch(&[&sdf_a, &sdf_b], &style);
        assert_eq!(result.len(), 2);
    }
}

//! Game text primitives — SDF-based text effects for game UI
//!
//! Applies visual effects (outline, shadow, glow, gradient) to SDF
//! glyphs using distance-field evaluation. Each effect is computed
//! per-pixel from the signed distance value, then composited via
//! Porter-Duff "over" blending.
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use crate::glyph::{GlyphSdf, GLYPH_SDF_SIZE};

/// RGBA color (16 bytes, #[repr(C)])
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Linear interpolation
    #[inline]
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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

/// SDF-based text effect.
///
/// The five original variants (`Outline` / `Shadow` / `Glow` / `InnerShadow`
/// / `Gradient`) are CPU-implemented in [`style_glyph`] and produce
/// bit-identical output on any target — safe to bake into pre-rendered
/// atlases.
///
/// The R.3-c variants added in 2026-07-29 (`Dissolve` .. `Liquid`) are
/// **spec-only** representations for GPU-shader consumption. `style_glyph`
/// falls through to `Color4::TRANSPARENT` for these — the intent is that a
/// GPU renderer reads the spec, sets shader uniforms, and animates
/// per-frame. Storing the spec here (rather than in each site's block
/// schema) keeps ALICE-Font as the one source of Font effect definitions.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "snake_case"))]
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

    // ═══ R.3-c variants ═══
    //
    // GPU-only specs: `style_glyph` returns Color4::TRANSPARENT for these
    // (documented above). Consumers read the parameters, set shader
    // uniforms, and animate per frame.
    /// Noise-based erosion. `t` is 0..1 progress (0 = fully solid,
    /// 1 = fully dissolved). `scale` controls noise frequency.
    Dissolve { t: f32, scale: f32 },
    /// Radial breathing distortion. `amount` = displacement magnitude,
    /// `frequency` = temporal cycles per second.
    DistortionBulge { amount: f32, frequency: f32 },
    /// Concentric halo rings around the glyph.
    HaloRings {
        color: Color4,
        /// Radial distance between successive rings (SDF units).
        spacing: f32,
        /// How many rings to emit (1..=8).
        count: u8,
    },
    /// Wavy edge animation. `amount` = wave amplitude.
    EdgeRipple { amount: f32 },
    /// SDF dilation animation — letters swell / melt over time.
    MeltDilate { amount: f32 },
    /// Water ripple + droplet effect (the `uWaterRipple*` uniform family).
    WaterRipple {
        color: Color4,
        amplitude: f32,
        frequency: f32,
        speed: f32,
        reach: f32,
        y_squash: f32,
        droplet_amp: f32,
    },
    /// Metaball collision — up to 3 spheres influencing the glyph.
    MetaballCollision {
        tint: Color4,
        influence: f32,
        /// XY of each metaball in the same NDC space as the text.
        balls: [[f32; 2]; 3],
        /// How many of the `balls` entries are live (0..=3). Extras are
        /// treated as inactive by the shader.
        active_count: u8,
    },
    /// Chromatic aberration (RGB split).
    ChromaticAberration { amount: f32 },
    /// SDF decompose — letter-shatter animation.
    Decompose { color: Color4, amount: f32 },
    /// Gravity-lens distortion (light-warping around a virtual mass).
    GravityLens { color: Color4, amount: f32 },
    /// Tendrils spawning outward from the glyph.
    Tendrils { color: Color4, amount: f32 },
    /// Sumi ink (墨) — fluid-driven bleed / kasure (掠れ) / splash /
    /// wash. Each parameter is 0..1 intensity.
    Sumi {
        color: Color4,
        bleed: f32,
        kasure: f32,
        splash: f32,
        wash: f32,
    },
    /// Cursor-following parallax. `cursor_ndc` is updated per frame by
    /// the caller.
    Parallax { amount: f32, cursor_ndc: [f32; 2] },
    /// Sinusoidal text warp displacement.
    TextWarp { frequency: f32, amplitude: f32 },
    /// Liquid morph — smooth per-pixel displacement.
    Liquid { amplitude: f32, frequency: f32 },

    // ═══ R.3-d — 9-technique checklist variants. Each corresponds to one
    // glyph-shader operator, so an ALICE-Font-aware renderer can compose
    // them into the glyph shader.
    /// #2 — FBM displacement bevel around the glyph edge. Produces the
    /// "carved paper" look documented in §12.6.
    Fbm2Bevel { amplitude: f32, frequency: f32 },
    /// #2 亜種 — Gyroid microstructure texture layered over the glyph
    /// interior (triply-periodic minimal surface).
    GyroidMicrostructure { scale: f32, thickness: f32 },
    /// #2 亜種 — Voronoi crack pattern applied to the glyph edge.
    VoronoiCrack { scale: f32, gap: f32 },
    /// #9 — Beer-Lambert absorption for translucent thick glyphs. The
    /// three absorb channels attenuate their RGB independently.
    BeerLambertAbsorption { absorb: Color4, thickness: f32 },
    /// #5 — iq analytic 2D primitive flourish around the glyph edge
    /// (Star / Heart / Diamond / Cross).
    IqFlourish {
        color: Color4,
        primitive: IqPrimitive,
        /// Radius of a single decorative primitive in SDF units.
        radius: f32,
        /// How many copies to arrange around the glyph (1..=32).
        count: u8,
    },

    // ═══ R.3-g — Keijiro Kino effects (2026-07-29 Phase 2 absorbed) ═══
    //
    // Ported (as SDF/screen-space concept) from github.com/keijiro/Kino*
    // (all MIT or Unlicense). Copyright 2015-2019 Keijiro Takahashi.
    // See LICENSE-KINO for full attribution. These are GPU-only specs;
    // `style_glyph` returns `Color4::TRANSPARENT` for consumers to read
    // the parameters and drive shader uniforms.
    /// Video codec artifact simulation (block-hold + color quantize +
    /// per-block noisy offset). Ported from `keijiro/KinoDatamosh`.
    Datamosh {
        /// 0..1 effect strength (mix vs. clean SDF).
        intensity: f32,
        /// Pixel block size for the hold-and-quantize grid (4..32 typical).
        block_size: f32,
        /// Number of color quantization steps (2..256).
        color_steps: u8,
        /// Per-glyph deterministic seed for block offsets.
        seed: u32,
    },
    /// Sobel / Laplacian edge detection on the SDF field. Ported from
    /// `keijiro/KinoContour`. Distinct from `Outline`: `Contour` uses
    /// gradient magnitude to emphasize surface curvature/edges, not
    /// silhouette expansion.
    Contour {
        color: Color4,
        /// Line width in SDF units (0.005..0.05 typical).
        thickness: f32,
        /// SDF gradient magnitude threshold to trigger a contour pixel
        /// (0.1..1.0 typical).
        threshold: f32,
    },
    /// Anamorphic lens streak — horizontal (or angled) light streak
    /// bleeding from bright pixels. Ported from `keijiro/KinoStreak`.
    AnamorphicStreak {
        color: Color4,
        /// Streak half-length in SDF/screen units (0.05..0.5 typical).
        length: f32,
        /// Streak orientation in radians (0 = horizontal, PI/2 = vertical).
        angle: f32,
        /// Brightness threshold above which streak activates.
        threshold: f32,
    },
    /// Distance-based tint using a virtual depth coordinate. Ported from
    /// `keijiro/KinoFog`. For 2D text, depth is derived from screen
    /// position or a caller-supplied uniform.
    VolumetricFog {
        color: Color4,
        /// Depth (0..1) at which fog begins.
        near: f32,
        /// Depth at which fog fully occludes.
        far: f32,
        /// Fog density coefficient (0..1).
        density: f32,
    },
    /// Isoline (contour lines at fixed SDF-value intervals). Ported from
    /// `keijiro/KinoIsoline`. Creates parallel "topographic" bands on the
    /// glyph interior/exterior.
    Isoline {
        color: Color4,
        /// Distance between successive isolines in SDF units.
        spacing: f32,
        /// Isoline width in SDF units.
        thickness: f32,
    },

    // ═══ R.3-h — Original ALICE-Font effects (2026-07-30 v0.3.3) ═══
    //
    // Fragment-level effects that TTF / OTF / Variable Fonts explicitly
    // cannot achieve. Written from scratch (not ported from prior art)
    // — the goal is a clear "TTF cannot do this" demonstration.
    /// Gravity-driven candlewax / molasses melt. The SDF field is
    /// displaced downward with per-column noise trails so the glyph
    /// actually drips over time. Distinct from `MeltDilate` which only
    /// oscillates edges.
    MeltDrip {
        /// Melt progress 0..1 (0 = solid glyph, 1 = fully melted puddle).
        progress: f32,
        /// Drip trail width in SDF units (0.01..0.1 typical).
        viscosity: f32,
        /// Drip tint (typically matches the glyph colour; alpha < 1 for
        /// translucent wax).
        color: Color4,
    },

    /// Real-time parametric weight interpolation. The same 40-byte
    /// `MetaFontParams` gets its `weight` axis modulated per fragment,
    /// which is impossible in TTF (baked outlines) and even in Variable
    /// Fonts (whose interpolation happens at the outline / shape-glyph
    /// level, not per-fragment).
    WeightMorph {
        /// Minimum weight (0.0 = hairline).
        min_weight: f32,
        /// Maximum weight (1.0 = black).
        max_weight: f32,
        /// Oscillation frequency in Hz.
        frequency: f32,
    },

    /// Domain-level SDF slicing. The glyph is split into N cells along
    /// an axis (vertical / horizontal / grid) and each cell is displaced
    /// outward to expose the SDF cross-section, then reconvenes. This
    /// is a runtime CSG operation on the glyph that TTF / OTF cannot do:
    /// the underlying outline data is static, so no per-frame slicing
    /// is expressible in the fontfile format.
    AxisSlice {
        /// 0.0 = vertical cut only, 1.0 = horizontal cut only, mixed values
        /// interpolate. -1.0 = both axes simultaneously (grid slice).
        axis: f32,
        /// Number of slices per axis (1 = single cut, 4 = 4×4 grid).
        count: u8,
        /// Oscillation frequency in Hz (drives open ↔ close animation).
        frequency: f32,
        /// Maximum gap width in UV units when fully open (0.02..0.20).
        gap_width: f32,
        /// Cross-section highlight colour (bright to expose "cut" edge).
        gap_color: Color4,
    },

    /// Fluid-density coalescence. The character shape emerges from an
    /// external fluid density field (e.g. a Navier-Stokes solver)
    /// filtered by the glyph SDF acting as an attractor mask. Empty
    /// space → fluid flows in → character forms. Distinct from `Sumi`
    /// which paints ink effects on a pre-existing static glyph — here
    /// the glyph literally does not appear until enough fluid has
    /// collected inside its SDF interior. No static font can express
    /// "shape emerges from external dynamic state".
    FluidForm {
        /// 0..1 intensity — 0 = never appears, 1 = fully requires fluid.
        intensity: f32,
        /// Attractor strength (higher = fluid gathers into character
        /// faster). Typical 0.5..2.0.
        attractor: f32,
        /// Ink/liquid colour that fills the character as fluid coalesces.
        color: Color4,
    },

    // ═══ R.3-i — physics-driven variants ═══
    //
    // Physics-driven destruction / combustion / melt effects that pair
    // real fluid (Navier-Stokes) and rigid-body (sub-piece) simulation
    // with SDF-based glyph modulation (`destructionAt` / `sdDestruction` /
    // `sdDebris` / `voronoi2`).
    // GPU-only specs — `style_glyph` returns `Color4::TRANSPARENT`.
    /// Combustion driven by an external heat field (Navier-Stokes fluid
    /// density) with SDF edge erosion and viridis-interpolated fire
    /// palette. `impacts[0..active_count]` are ignition points injected
    /// into the heat field per frame.
    Burn {
        /// Base heat level (0..1) — non-impact background combustion.
        intensity: f32,
        /// Ignition points. `Impact::radius` is treated as ignition
        /// strength (fluid density injected) rather than a spatial radius.
        impacts: [Impact; MAX_IMPACTS],
        /// How many of the `impacts` entries are live (0..=MAX_IMPACTS).
        active_count: u8,
    },
    /// SDF-based shattering with Voronoi cracks + rigid sub-piece
    /// physics. Each impact spawns an expanding wavefront + core removal
    /// pattern seeded by `seed` so consecutive clicks give different
    /// crack patterns.
    Shatter {
        /// Impact points (spatial radius controls destruction cell scale).
        impacts: [Impact; MAX_IMPACTS],
        /// How many of the `impacts` entries are live (0..=MAX_IMPACTS).
        active_count: u8,
        /// Per-effect deterministic pattern seed (typically frame-random).
        seed: u32,
    },
    /// Total destruction: Shatter + canonical `sdDebris` 3D-ish chunk
    /// pattern + thermal glow (Beer-Lambert absorption) + viridis heat
    /// palette. Geometrically distinct from `Shatter` — `sdDebris`
    /// emits chunks whose surface carries fbm displacement while
    /// `Shatter` only removes cells from the flat glyph field.
    Destroy {
        /// Impact points.
        impacts: [Impact; MAX_IMPACTS],
        /// How many of the `impacts` entries are live (0..=MAX_IMPACTS).
        active_count: u8,
        /// Per-effect deterministic pattern seed.
        seed: u32,
        /// Thermal overlay strength (0..1) — 0 = no heat glow, 1 = full
        /// molten-edge Beer-Lambert absorption on debris interior.
        thermal: f32,
    },
}

/// Maximum number of concurrent impact points that a physics-driven
/// [`TextEffect`] variant ([`TextEffect::Burn`] / [`TextEffect::Shatter`]
/// / [`TextEffect::Destroy`]) can hold. Chosen to match the corresponding
/// `MAX_IMPACTS` GLSL constant in the consumer shader.
pub const MAX_IMPACTS: usize = 8;

/// Single impact point for physics-driven [`TextEffect`] variants
/// (`Burn` / `Shatter` / `Destroy`).
///
/// * `x`, `y` — impact position in the same NDC / canvas space the
///   caller uses to place the text.
/// * `radius` — spatial influence radius (for `Shatter` / `Destroy`)
///   or ignition strength (for `Burn`) — units are consumer-defined.
/// * `seed` — per-impact deterministic seed (0..1); used by the
///   Voronoi crack shader to give each click a different pattern.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Impact {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub seed: f32,
}

impl Impact {
    /// Empty impact slot (all zeros) — used as the default value when
    /// building an `[Impact; MAX_IMPACTS]` array with fewer than
    /// `MAX_IMPACTS` live entries.
    pub const EMPTY: Self = Self {
        x: 0.0,
        y: 0.0,
        radius: 0.0,
        seed: 0.0,
    };
}

/// Analytic 2D SDF primitives sourced from iq shadertoy — used by
/// [`TextEffect::IqFlourish`] to decorate glyph edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum IqPrimitive {
    /// Five-pointed star (`sdStar5` in alice_operators.glsl).
    Star5,
    /// Heart (`sdHeart`).
    Heart,
    /// Diamond / rhombus (`sdDiamond`).
    Diamond,
    /// Plus-sign cross (`sdCross`).
    Cross,
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
    #[must_use]
    pub const fn new() -> Self {
        Self {
            effects: [None; MAX_EFFECTS],
            count: 0,
        }
    }

    pub const fn push(&mut self, effect: TextEffect) -> bool {
        if self.count < MAX_EFFECTS {
            self.effects[self.count] = Some(effect);
            self.count += 1;
            true
        } else {
            false
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.count
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[must_use]
    pub const fn get(&self, index: usize) -> Option<&TextEffect> {
        if index < self.count {
            self.effects[index].as_ref()
        } else {
            None
        }
    }

    pub const fn clear(&mut self) {
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
    #[must_use]
    pub const fn default_style() -> Self {
        Self {
            base_color: Color4::WHITE,
            threshold: 0.0,
            aa_width: 0.03,
            effects: EffectStack::new(),
        }
    }

    /// White text with black outline
    #[must_use]
    pub const fn outlined() -> Self {
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
    #[must_use]
    pub const fn shadowed() -> Self {
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
    #[must_use]
    pub const fn neon() -> Self {
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
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
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
    let t = crate::law_float::law_clamp_f32(t, 0.0, 1.0);
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
                let radial_t = crate::law_float::law_min_f32(cu * cu + cv * cv, 0.25) * 4.0;
                let t = linear_t * (1.0 - radial_mix) + radial_t * radial_mix;
                let t = crate::law_float::law_clamp_f32(t, 0.0, 1.0);
                color_bottom.lerp(color_top, t)
            }
        }
        // ─── R.3-c GPU-only spec variants ─────────────────────────
        // These are per-frame animated effects driven by a GPU shader.
        // CPU rasterisation returns transparent so the callers who bake
        // static atlases don't accidentally embed frame-0 samples of an
        // animation. GPU consumers should read the spec directly.
        TextEffect::Dissolve { .. }
        | TextEffect::DistortionBulge { .. }
        | TextEffect::HaloRings { .. }
        | TextEffect::EdgeRipple { .. }
        | TextEffect::MeltDilate { .. }
        | TextEffect::WaterRipple { .. }
        | TextEffect::MetaballCollision { .. }
        | TextEffect::ChromaticAberration { .. }
        | TextEffect::Decompose { .. }
        | TextEffect::GravityLens { .. }
        | TextEffect::Tendrils { .. }
        | TextEffect::Sumi { .. }
        | TextEffect::Parallax { .. }
        | TextEffect::TextWarp { .. }
        | TextEffect::Liquid { .. }
        // R.3-d — 9-technique variants (GPU-only spec, same fallthrough)
        | TextEffect::Fbm2Bevel { .. }
        | TextEffect::GyroidMicrostructure { .. }
        | TextEffect::VoronoiCrack { .. }
        | TextEffect::BeerLambertAbsorption { .. }
        | TextEffect::IqFlourish { .. }
        // R.3-g — Keijiro Kino effects (GPU-only spec, same fallthrough)
        | TextEffect::Datamosh { .. }
        | TextEffect::Contour { .. }
        | TextEffect::AnamorphicStreak { .. }
        | TextEffect::VolumetricFog { .. }
        | TextEffect::Isoline { .. }
        // R.3-h — Original ALICE-Font effects (GPU-only spec)
        | TextEffect::MeltDrip { .. }
        | TextEffect::WeightMorph { .. }
        | TextEffect::AxisSlice { .. }
        | TextEffect::FluidForm { .. }
        // R.3-i — physics-driven, GPU-only
        | TextEffect::Burn { .. }
        | TextEffect::Shatter { .. }
        | TextEffect::Destroy { .. } => {
            // Silence unused args for the GPU-only path.
            let _ = (d, u, v, sdf, aa);
            Color4::TRANSPARENT
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
        let m = f32::from_bits((bits & 0x007F_FFFF) | 0x3F80_0000) - 1.0;
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
#[must_use]
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
#[must_use]
pub fn style_glyphs_batch(
    sdfs: &[&GlyphSdf],
    style: &GameTextStyle,
) -> alloc::vec::Vec<StyledGlyph> {
    sdfs.iter().map(|sdf| style_glyph(sdf, style)).collect()
}

#[cfg(not(feature = "std"))]
#[must_use]
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

    #[cfg(feature = "serde")]
    #[test]
    fn r3c_r3d_variants_serde_roundtrip() {
        // Sample one variant from each R.3-c / R.3-d group and confirm
        // the tag+params JSON schema roundtrips without loss. This gates
        // schema drift when a shader-side rename happens.
        let samples = [
            TextEffect::Dissolve { t: 0.4, scale: 8.0 },
            TextEffect::HaloRings {
                color: Color4::new(1.0, 0.8, 0.3, 1.0),
                spacing: 0.05,
                count: 3,
            },
            TextEffect::WaterRipple {
                color: Color4::new(0.3, 0.6, 0.9, 0.7),
                amplitude: 0.02,
                frequency: 25.0,
                speed: 2.5,
                reach: 0.3,
                y_squash: 0.15,
                droplet_amp: 0.4,
            },
            TextEffect::MetaballCollision {
                tint: Color4::new(0.4, 0.7, 1.0, 1.0),
                influence: 0.8,
                balls: [[0.1, 0.2], [-0.3, 0.4], [0.0, 0.0]],
                active_count: 2,
            },
            TextEffect::Sumi {
                color: Color4::new(0.05, 0.05, 0.08, 1.0),
                bleed: 0.6,
                kasure: 0.3,
                splash: 0.2,
                wash: 0.5,
            },
            TextEffect::Fbm2Bevel {
                amplitude: 0.02,
                frequency: 4.0,
            },
            TextEffect::GyroidMicrostructure {
                scale: 20.0,
                thickness: 0.05,
            },
            TextEffect::VoronoiCrack {
                scale: 8.0,
                gap: 0.03,
            },
            TextEffect::BeerLambertAbsorption {
                absorb: Color4::new(2.5, 2.5, 2.2, 1.0),
                thickness: 0.5,
            },
            TextEffect::IqFlourish {
                color: Color4::new(0.9, 0.7, 0.2, 1.0),
                primitive: IqPrimitive::Star5,
                radius: 0.03,
                count: 8,
            },
            // R.3-g — Keijiro Kino effects (5 variants)
            TextEffect::Datamosh {
                intensity: 0.7,
                block_size: 8.0,
                color_steps: 16,
                seed: 0xdeadbeef,
            },
            TextEffect::Contour {
                color: Color4::new(0.9, 0.9, 0.9, 1.0),
                thickness: 0.02,
                threshold: 0.4,
            },
            TextEffect::AnamorphicStreak {
                color: Color4::new(1.0, 0.9, 0.8, 1.0),
                length: 0.25,
                angle: 0.0,
                threshold: 0.6,
            },
            TextEffect::VolumetricFog {
                color: Color4::new(0.5, 0.55, 0.6, 1.0),
                near: 0.2,
                far: 0.9,
                density: 0.6,
            },
            TextEffect::Isoline {
                color: Color4::new(0.2, 0.8, 0.5, 1.0),
                spacing: 0.12,
                thickness: 0.01,
            },
            // R.3-h — Original ALICE-Font effects (v0.3.3)
            TextEffect::MeltDrip {
                progress: 0.6,
                viscosity: 0.04,
                color: Color4::new(0.95, 0.7, 0.35, 1.0),
            },
            TextEffect::WeightMorph {
                min_weight: 0.3,
                max_weight: 0.9,
                frequency: 0.5,
            },
            TextEffect::AxisSlice {
                axis: 0.0,
                count: 3,
                frequency: 0.8,
                gap_width: 0.12,
                gap_color: Color4::new(1.0, 0.5, 0.2, 1.0),
            },
            TextEffect::FluidForm {
                intensity: 0.85,
                attractor: 1.5,
                color: Color4::new(0.02, 0.02, 0.05, 1.0),
            },
        ];
        for e in &samples {
            let json = serde_json::to_string(e).expect("serialise");
            let back: TextEffect = serde_json::from_str(&json).expect("deserialise");
            // Round-trip once more to catch schema-level drift; the two
            // JSON strings must be identical.
            let json2 = serde_json::to_string(&back).expect("serialise round 2");
            assert_eq!(
                json, json2,
                "roundtrip mismatch for {e:?}: {json} vs {json2}"
            );
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn text_effect_kind_tag_is_snake_case() {
        // Consumers parsing the JSON schema (block schemas etc.)
        // rely on `{ "kind": "dissolve", ... }` shape rather than
        // `{ "Dissolve": ... }`.
        let e = TextEffect::Dissolve { t: 0.5, scale: 4.0 };
        let json = serde_json::to_string(&e).unwrap();
        assert!(
            json.contains("\"kind\":\"dissolve\""),
            "expected snake_case kind tag, got {json}"
        );
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

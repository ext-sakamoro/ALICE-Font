//! MetaFont parameters — 40-byte parametric font descriptor
//!
//! 10 floating-point values that fully describe a typeface.
//! Any weight, width, serif style, and italic angle can be
//! expressed as a point in this 10-dimensional space.
//!
//! License: MIT
//! Author: Moroya Sakamoto

/// Parametric font descriptor (40 bytes)
///
/// Encodes a complete typeface as 10 f32 parameters.
/// Transmitted over network instead of multi-MB TTF/WOFF files.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct MetaFontParams {
    /// Stroke weight: 0.0 = hairline, 0.5 = regular, 1.0 = black
    pub weight: f32,
    /// Horizontal width: 0.5 = condensed, 1.0 = normal, 1.5 = extended
    pub width: f32,
    /// Serif amount: 0.0 = sans-serif, 1.0 = full serif
    pub serif: f32,
    /// Thick/thin contrast: 0.0 = monoweight, 1.0 = high contrast
    pub contrast: f32,
    /// Italic slant angle in radians (0.0 = upright, ~0.2 = italic)
    pub slant: f32,
    /// x-height ratio relative to em (0.4-0.6 typical)
    pub x_height: f32,
    /// Capital height ratio relative to em
    pub cap_height: f32,
    /// Ascender ratio relative to em
    pub ascender: f32,
    /// Descender ratio relative to em (positive = below baseline)
    pub descender: f32,
    /// Corner roundness: 0.0 = sharp, 1.0 = fully rounded
    pub roundness: f32,
}

impl MetaFontParams {
    /// Wire size in bytes
    pub const SIZE: usize = 40;

    /// Sans-serif regular (like Helvetica / Arial)
    pub const fn sans_regular() -> Self {
        Self {
            weight: 0.45,
            width: 1.0,
            serif: 0.0,
            contrast: 0.15,
            slant: 0.0,
            x_height: 0.52,
            cap_height: 0.72,
            ascender: 0.80,
            descender: 0.22,
            roundness: 0.3,
        }
    }

    /// Sans-serif bold
    pub const fn sans_bold() -> Self {
        Self {
            weight: 0.75,
            width: 1.02,
            serif: 0.0,
            contrast: 0.10,
            slant: 0.0,
            x_height: 0.54,
            cap_height: 0.72,
            ascender: 0.80,
            descender: 0.22,
            roundness: 0.3,
        }
    }

    /// Serif regular (like Times New Roman)
    pub const fn serif_regular() -> Self {
        Self {
            weight: 0.42,
            width: 1.0,
            serif: 0.8,
            contrast: 0.55,
            slant: 0.0,
            x_height: 0.45,
            cap_height: 0.68,
            ascender: 0.78,
            descender: 0.22,
            roundness: 0.1,
        }
    }

    /// Serif italic
    pub const fn serif_italic() -> Self {
        Self {
            weight: 0.42,
            width: 0.98,
            serif: 0.7,
            contrast: 0.55,
            slant: 0.21,
            x_height: 0.45,
            cap_height: 0.68,
            ascender: 0.78,
            descender: 0.22,
            roundness: 0.1,
        }
    }

    /// Monospace regular (like Courier)
    pub const fn mono_regular() -> Self {
        Self {
            weight: 0.40,
            width: 0.6,
            serif: 0.5,
            contrast: 0.0,
            slant: 0.0,
            x_height: 0.53,
            cap_height: 0.70,
            ascender: 0.80,
            descender: 0.25,
            roundness: 0.0,
        }
    }

    /// Gothic / display (like Impact)
    pub const fn display_heavy() -> Self {
        Self {
            weight: 0.95,
            width: 0.7,
            serif: 0.0,
            contrast: 0.05,
            slant: 0.0,
            x_height: 0.60,
            cap_height: 0.75,
            ascender: 0.80,
            descender: 0.18,
            roundness: 0.1,
        }
    }

    /// Interpolate between two font parameter sets
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            weight: self.weight + (other.weight - self.weight) * t,
            width: self.width + (other.width - self.width) * t,
            serif: self.serif + (other.serif - self.serif) * t,
            contrast: self.contrast + (other.contrast - self.contrast) * t,
            slant: self.slant + (other.slant - self.slant) * t,
            x_height: self.x_height + (other.x_height - self.x_height) * t,
            cap_height: self.cap_height + (other.cap_height - self.cap_height) * t,
            ascender: self.ascender + (other.ascender - self.ascender) * t,
            descender: self.descender + (other.descender - self.descender) * t,
            roundness: self.roundness + (other.roundness - self.roundness) * t,
        }
    }

    /// Encode to 40-byte wire format (little-endian)
    pub fn encode(&self) -> [u8; 40] {
        let mut buf = [0u8; 40];
        let fields = [
            self.weight,
            self.width,
            self.serif,
            self.contrast,
            self.slant,
            self.x_height,
            self.cap_height,
            self.ascender,
            self.descender,
            self.roundness,
        ];
        for (i, f) in fields.iter().enumerate() {
            let bytes = f.to_le_bytes();
            buf[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        }
        buf
    }

    /// Decode from 40-byte wire format (little-endian)
    pub fn decode(data: &[u8; 40]) -> Self {
        let mut fields = [0.0f32; 10];
        for i in 0..10 {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[i * 4..i * 4 + 4]);
            fields[i] = f32::from_le_bytes(bytes);
        }
        Self {
            weight: fields[0],
            width: fields[1],
            serif: fields[2],
            contrast: fields[3],
            slant: fields[4],
            x_height: fields[5],
            cap_height: fields[6],
            ascender: fields[7],
            descender: fields[8],
            roundness: fields[9],
        }
    }

    /// Actual stroke half-width for rendering (in em units)
    pub fn stroke_half_width(&self) -> f32 {
        0.01 + self.weight * 0.08
    }

    /// Thick stroke half-width (for contrast)
    pub fn thick_half_width(&self) -> f32 {
        self.stroke_half_width() * (1.0 + self.contrast * 0.8)
    }

    /// Thin stroke half-width (for contrast)
    pub fn thin_half_width(&self) -> f32 {
        self.stroke_half_width() * (1.0 - self.contrast * 0.5)
    }

    /// Serif bracket length
    pub fn serif_length(&self) -> f32 {
        self.serif * 0.06
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_size() {
        assert_eq!(core::mem::size_of::<MetaFontParams>(), 40);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let params = MetaFontParams::sans_regular();
        let encoded = params.encode();
        assert_eq!(encoded.len(), 40);
        let decoded = MetaFontParams::decode(&encoded);
        assert!((decoded.weight - params.weight).abs() < 1e-6);
        assert!((decoded.serif - params.serif).abs() < 1e-6);
        assert!((decoded.slant - params.slant).abs() < 1e-6);
    }

    #[test]
    fn test_presets() {
        let sans = MetaFontParams::sans_regular();
        assert!((sans.serif - 0.0).abs() < 0.01);

        let serif = MetaFontParams::serif_regular();
        assert!(serif.serif > 0.5);

        let bold = MetaFontParams::sans_bold();
        assert!(bold.weight > sans.weight);
    }

    #[test]
    fn test_lerp() {
        let a = MetaFontParams::sans_regular();
        let b = MetaFontParams::sans_bold();
        let mid = a.lerp(&b, 0.5);
        assert!((mid.weight - (a.weight + b.weight) / 2.0).abs() < 0.01);
    }

    #[test]
    fn test_stroke_widths() {
        let params = MetaFontParams::serif_regular();
        let thick = params.thick_half_width();
        let thin = params.thin_half_width();
        assert!(thick > thin); // high contrast → thick > thin
    }

    #[test]
    fn test_mono_no_contrast() {
        let mono = MetaFontParams::mono_regular();
        assert!((mono.contrast - 0.0).abs() < 0.01);
        let thick = mono.thick_half_width();
        let thin = mono.thin_half_width();
        assert!((thick - thin).abs() < 0.01);
    }

    #[test]
    fn test_serif_length() {
        let sans = MetaFontParams::sans_regular();
        assert!((sans.serif_length()).abs() < 0.001);
        let serif = MetaFontParams::serif_regular();
        assert!(serif.serif_length() > 0.01);
    }

    #[test]
    fn test_display_heavy() {
        let d = MetaFontParams::display_heavy();
        assert!(d.weight > 0.9);
        assert!(d.width < 0.8); // condensed
    }
}

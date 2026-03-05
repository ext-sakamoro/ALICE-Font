//! C-ABI FFI — `#[no_mangle] pub extern "C"` functions for Unity/UE5
//!
//! All functions prefixed with `aa_font_` (ALICE API convention).
//! Enabled via `--features ffi`.
//!
//! License: MIT
//! Author: Moroya Sakamoto

use crate::game::{style_glyph, GameTextStyle};
use crate::glyph::{GlyphGenerator, GlyphSdf, GLYPH_SDF_SIZE};
use crate::license::{FontLicense, LicenseValidator, PlatformRestriction, ValidationResult};
use crate::param::MetaFontParams;

// ---------------------------------------------------------------------------
// MetaFontParams
// ---------------------------------------------------------------------------

/// Create sans-regular preset. Caller owns the returned value.
#[no_mangle]
pub extern "C" fn aa_font_params_sans_regular() -> MetaFontParams {
    MetaFontParams::sans_regular()
}

/// Create sans-bold preset.
#[no_mangle]
pub extern "C" fn aa_font_params_sans_bold() -> MetaFontParams {
    MetaFontParams::sans_bold()
}

/// Create serif-regular preset.
#[no_mangle]
pub extern "C" fn aa_font_params_serif_regular() -> MetaFontParams {
    MetaFontParams::serif_regular()
}

/// Create serif-italic preset.
#[no_mangle]
pub extern "C" fn aa_font_params_serif_italic() -> MetaFontParams {
    MetaFontParams::serif_italic()
}

/// Create mono-regular preset.
#[no_mangle]
pub extern "C" fn aa_font_params_mono_regular() -> MetaFontParams {
    MetaFontParams::mono_regular()
}

/// Create display-heavy preset.
#[no_mangle]
pub extern "C" fn aa_font_params_display_heavy() -> MetaFontParams {
    MetaFontParams::display_heavy()
}

/// Interpolate between two parameter sets.
#[no_mangle]
pub extern "C" fn aa_font_params_lerp(
    a: &MetaFontParams,
    b: &MetaFontParams,
    t: f32,
) -> MetaFontParams {
    a.lerp(b, t)
}

/// Encode parameters to 40-byte wire format. Writes to `out` (must be >= 40 bytes).
///
/// # Safety
/// `out` must point to at least 40 bytes of writable memory.
#[no_mangle]
pub unsafe extern "C" fn aa_font_params_encode(params: &MetaFontParams, out: *mut u8) {
    let encoded = params.encode();
    core::ptr::copy_nonoverlapping(encoded.as_ptr(), out, 40);
}

/// Decode parameters from 40-byte wire format.
///
/// # Safety
/// `data` must point to at least 40 bytes of readable memory.
#[no_mangle]
pub unsafe extern "C" fn aa_font_params_decode(data: *const u8) -> MetaFontParams {
    let mut buf = [0u8; 40];
    core::ptr::copy_nonoverlapping(data, buf.as_mut_ptr(), 40);
    MetaFontParams::decode(&buf)
}

/// Get stroke half-width for given params.
#[no_mangle]
pub extern "C" fn aa_font_params_stroke_half_width(params: &MetaFontParams) -> f32 {
    params.stroke_half_width()
}

/// Get thick half-width for given params.
#[no_mangle]
pub extern "C" fn aa_font_params_thick_half_width(params: &MetaFontParams) -> f32 {
    params.thick_half_width()
}

/// Get thin half-width for given params.
#[no_mangle]
pub extern "C" fn aa_font_params_thin_half_width(params: &MetaFontParams) -> f32 {
    params.thin_half_width()
}

/// Get serif length for given params.
#[no_mangle]
pub extern "C" fn aa_font_params_serif_length(params: &MetaFontParams) -> f32 {
    params.serif_length()
}

// ---------------------------------------------------------------------------
// GlyphGenerator / GlyphSdf
// ---------------------------------------------------------------------------

/// Create a glyph generator. Returns heap-allocated pointer. Caller must free with `aa_font_generator_free`.
#[no_mangle]
pub extern "C" fn aa_font_generator_new(params: &MetaFontParams) -> *mut GlyphGenerator {
    let gen = GlyphGenerator::new(params);
    alloc::boxed::Box::into_raw(alloc::boxed::Box::new(gen))
}

/// Free a glyph generator.
///
/// # Safety
/// `ptr` must be a valid pointer returned by `aa_font_generator_new`.
#[no_mangle]
pub unsafe extern "C" fn aa_font_generator_free(ptr: *mut GlyphGenerator) {
    if !ptr.is_null() {
        drop(alloc::boxed::Box::from_raw(ptr));
    }
}

/// Generate SDF for a character. Returns heap-allocated pointer. Caller must free with `aa_font_glyph_free`.
///
/// # Safety
/// `gen` must be a valid pointer from `aa_font_generator_new`.
#[no_mangle]
pub unsafe extern "C" fn aa_font_generator_generate(
    gen: *const GlyphGenerator,
    ch: u8,
) -> *mut GlyphSdf {
    let gen = &*gen;
    let sdf = gen.generate(ch);
    alloc::boxed::Box::into_raw(alloc::boxed::Box::new(sdf))
}

/// Free a glyph SDF.
///
/// # Safety
/// `ptr` must be a valid pointer returned by `aa_font_generator_generate`.
#[no_mangle]
pub unsafe extern "C" fn aa_font_glyph_free(ptr: *mut GlyphSdf) {
    if !ptr.is_null() {
        drop(alloc::boxed::Box::from_raw(ptr));
    }
}

/// Sample SDF at normalized coordinates.
///
/// # Safety
/// `sdf` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn aa_font_glyph_sample(sdf: *const GlyphSdf, u: f32, v: f32) -> f32 {
    (*sdf).sample(u, v)
}

/// Check if point is inside glyph.
///
/// # Safety
/// `sdf` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn aa_font_glyph_is_inside(sdf: *const GlyphSdf, u: f32, v: f32) -> bool {
    (*sdf).is_inside(u, v)
}

/// Get glyph advance width.
///
/// # Safety
/// `sdf` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn aa_font_glyph_advance(sdf: *const GlyphSdf) -> f32 {
    (*sdf).advance
}

/// Get SDF data pointer (read-only). Returns pointer to GLYPH_SDF_SIZE*GLYPH_SDF_SIZE f32 values.
///
/// # Safety
/// `sdf` must be a valid pointer. Returned pointer is valid as long as `sdf` is alive.
#[no_mangle]
pub unsafe extern "C" fn aa_font_glyph_data_ptr(sdf: *const GlyphSdf) -> *const f32 {
    (*sdf).data.as_ptr()
}

/// Get GLYPH_SDF_SIZE constant.
#[no_mangle]
pub extern "C" fn aa_font_glyph_sdf_size() -> u32 {
    GLYPH_SDF_SIZE as u32
}

// ---------------------------------------------------------------------------
// FontLicense
// ---------------------------------------------------------------------------

/// Create a free parametric font license.
///
/// # Safety
/// `params_encoded` must point to at least 40 bytes.
#[no_mangle]
pub unsafe extern "C" fn aa_font_license_parametric_free(params_encoded: *const u8) -> FontLicense {
    let mut buf = [0u8; 40];
    core::ptr::copy_nonoverlapping(params_encoded, buf.as_mut_ptr(), 40);
    FontLicense::parametric_free(&buf)
}

/// Create a game-title license.
///
/// # Safety
/// `params_encoded` must point to at least 40 bytes.
#[no_mangle]
pub unsafe extern "C" fn aa_font_license_for_game(
    params_encoded: *const u8,
    title_id: u32,
    platforms: u8,
) -> FontLicense {
    let mut buf = [0u8; 40];
    core::ptr::copy_nonoverlapping(params_encoded, buf.as_mut_ptr(), 40);
    FontLicense::for_game_title(&buf, title_id, PlatformRestriction(platforms))
}

/// Validate a font license. Returns 0=Valid, 1=Expired, 2=PlatformDenied,
/// 3=RightDenied, 4=TitleMismatch, 5=SeatLimitExceeded, 6=ParamsMismatch.
///
/// # Safety
/// `params_encoded` must point to at least 40 bytes.
#[no_mangle]
pub unsafe extern "C" fn aa_font_license_validate(
    license: &FontLicense,
    current_epoch: u32,
    platform: u8,
    required_right: u16,
    title_id: u32,
    current_seats: u16,
    params_encoded: *const u8,
) -> u32 {
    let mut buf = [0u8; 40];
    core::ptr::copy_nonoverlapping(params_encoded, buf.as_mut_ptr(), 40);
    match LicenseValidator::validate(
        license,
        current_epoch,
        platform,
        required_right,
        title_id,
        current_seats,
        &buf,
    ) {
        ValidationResult::Valid => 0,
        ValidationResult::Expired => 1,
        ValidationResult::PlatformDenied => 2,
        ValidationResult::RightDenied => 3,
        ValidationResult::TitleMismatch => 4,
        ValidationResult::SeatLimitExceeded => 5,
        ValidationResult::ParamsMismatch => 6,
    }
}

/// Check if license is parametric free.
#[no_mangle]
pub extern "C" fn aa_font_license_is_parametric_free(license: &FontLicense) -> bool {
    LicenseValidator::is_parametric_free(license)
}

// ---------------------------------------------------------------------------
// GameTextStyle
// ---------------------------------------------------------------------------

/// Create default game text style. Returns heap-allocated pointer.
#[no_mangle]
pub extern "C" fn aa_font_style_default() -> *mut GameTextStyle {
    alloc::boxed::Box::into_raw(alloc::boxed::Box::new(GameTextStyle::default_style()))
}

/// Create outlined game text style.
#[no_mangle]
pub extern "C" fn aa_font_style_outlined() -> *mut GameTextStyle {
    alloc::boxed::Box::into_raw(alloc::boxed::Box::new(GameTextStyle::outlined()))
}

/// Create shadowed game text style.
#[no_mangle]
pub extern "C" fn aa_font_style_shadowed() -> *mut GameTextStyle {
    alloc::boxed::Box::into_raw(alloc::boxed::Box::new(GameTextStyle::shadowed()))
}

/// Create neon game text style.
#[no_mangle]
pub extern "C" fn aa_font_style_neon() -> *mut GameTextStyle {
    alloc::boxed::Box::into_raw(alloc::boxed::Box::new(GameTextStyle::neon()))
}

/// Free a game text style.
///
/// # Safety
/// `ptr` must be a valid pointer from `aa_font_style_*`.
#[no_mangle]
pub unsafe extern "C" fn aa_font_style_free(ptr: *mut GameTextStyle) {
    if !ptr.is_null() {
        drop(alloc::boxed::Box::from_raw(ptr));
    }
}

/// Style a glyph SDF. Writes RGBA f32 data to `out` (must be >= 32*32*4 floats = 16384 bytes).
/// Returns advance width.
///
/// # Safety
/// `sdf` must be valid. `style` must be valid. `out` must point to at least 4096 f32 values.
#[no_mangle]
pub unsafe extern "C" fn aa_font_style_glyph(
    sdf: *const GlyphSdf,
    style: *const GameTextStyle,
    out: *mut f32,
) -> f32 {
    let styled = style_glyph(&*sdf, &*style);
    let pixel_count = GLYPH_SDF_SIZE * GLYPH_SDF_SIZE;
    for i in 0..pixel_count {
        let c = &styled.pixels[i];
        *out.add(i * 4) = c.r;
        *out.add(i * 4 + 1) = c.g;
        *out.add(i * 4 + 2) = c.b;
        *out.add(i * 4 + 3) = c.a;
    }
    styled.advance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_params_presets() {
        let sans = aa_font_params_sans_regular();
        assert!(sans.weight > 0.0);
        let bold = aa_font_params_sans_bold();
        assert!(bold.weight > sans.weight);
        let serif = aa_font_params_serif_regular();
        assert!(serif.serif > 0.5);
        let italic = aa_font_params_serif_italic();
        assert!(italic.slant > 0.0);
        let mono = aa_font_params_mono_regular();
        assert!((mono.contrast).abs() < 0.01);
        let heavy = aa_font_params_display_heavy();
        assert!(heavy.weight > 0.9);
    }

    #[test]
    fn test_ffi_params_lerp() {
        let a = aa_font_params_sans_regular();
        let b = aa_font_params_sans_bold();
        let mid = aa_font_params_lerp(&a, &b, 0.5);
        assert!((mid.weight - (a.weight + b.weight) / 2.0).abs() < 0.01);
    }

    #[test]
    fn test_ffi_params_encode_decode() {
        let params = aa_font_params_sans_regular();
        let mut buf = [0u8; 40];
        unsafe {
            aa_font_params_encode(&params, buf.as_mut_ptr());
            let decoded = aa_font_params_decode(buf.as_ptr());
            assert!((decoded.weight - params.weight).abs() < 1e-6);
        }
    }

    #[test]
    fn test_ffi_params_widths() {
        let params = aa_font_params_serif_regular();
        let hw = aa_font_params_stroke_half_width(&params);
        let thick = aa_font_params_thick_half_width(&params);
        let thin = aa_font_params_thin_half_width(&params);
        assert!(thick > thin);
        assert!(hw > 0.0);
        let serif = aa_font_params_serif_length(&params);
        assert!(serif > 0.0);
    }

    #[test]
    fn test_ffi_generator_lifecycle() {
        let params = aa_font_params_sans_bold();
        let gen = aa_font_generator_new(&params);
        assert!(!gen.is_null());
        unsafe {
            let sdf = aa_font_generator_generate(gen, b'A');
            assert!(!sdf.is_null());
            let adv = aa_font_glyph_advance(sdf);
            assert!(adv > 0.0);
            let val = aa_font_glyph_sample(sdf, 0.5, 0.5);
            assert!(val.is_finite());
            let data = aa_font_glyph_data_ptr(sdf);
            assert!(!data.is_null());
            aa_font_glyph_free(sdf);
            aa_font_generator_free(gen);
        }
    }

    #[test]
    fn test_ffi_glyph_sdf_size() {
        assert_eq!(aa_font_glyph_sdf_size(), 32);
    }

    #[test]
    fn test_ffi_license_parametric() {
        let params = aa_font_params_sans_regular();
        let mut buf = [0u8; 40];
        unsafe {
            aa_font_params_encode(&params, buf.as_mut_ptr());
            let lic = aa_font_license_parametric_free(buf.as_ptr());
            assert!(aa_font_license_is_parametric_free(&lic));
            let result = aa_font_license_validate(&lic, 0, 0x01, 0x01, 0, 0, buf.as_ptr());
            assert_eq!(result, 0); // Valid
        }
    }

    #[test]
    fn test_ffi_style_lifecycle() {
        let style = aa_font_style_default();
        assert!(!style.is_null());
        unsafe {
            aa_font_style_free(style);
        }

        let style = aa_font_style_outlined();
        assert!(!style.is_null());
        unsafe {
            aa_font_style_free(style);
        }

        let style = aa_font_style_neon();
        assert!(!style.is_null());
        unsafe {
            aa_font_style_free(style);
        }
    }
}

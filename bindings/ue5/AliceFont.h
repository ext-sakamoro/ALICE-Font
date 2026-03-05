// ALICE-Font UE5 C++ Header
// extern "C" wrapper for alice_font FFI (C-ABI)
//
// Usage:
//   1. Build: cargo build --release --features ffi
//   2. Copy libalice_font.so / .dylib / .dll to ThirdParty/AliceFont/lib/
//   3. Include this header and link against the library
//
// License: MIT
// Author: Moroya Sakamoto

#pragma once

#include <cstdint>

extern "C"
{

// MetaFontParams — 40-byte parametric font descriptor (10 × f32)
struct AliceFontParams
{
    float weight;
    float width;
    float serif;
    float contrast;
    float slant;
    float x_height;
    float cap_height;
    float ascender;
    float descender;
    float roundness;
};

// FontLicense — 32-byte wire format
struct AliceFontLicense
{
    uint64_t content_hash;
    uint64_t params_hash;
    uint32_t title_id;
    uint32_t expires_epoch;
    uint16_t rights;
    uint16_t max_seats;
    uint8_t  license_type;
    uint8_t  platforms;
    uint8_t  reserved[2];
};

// Opaque handles
typedef void* AliceFontGenerator;
typedef void* AliceFontGlyphSdf;
typedef void* AliceFontGameTextStyle;

// --- MetaFontParams presets ---

AliceFontParams aa_font_params_sans_regular();
AliceFontParams aa_font_params_sans_bold();
AliceFontParams aa_font_params_serif_regular();
AliceFontParams aa_font_params_serif_italic();
AliceFontParams aa_font_params_mono_regular();
AliceFontParams aa_font_params_display_heavy();

AliceFontParams aa_font_params_lerp(
    const AliceFontParams* a, const AliceFontParams* b, float t);

void aa_font_params_encode(const AliceFontParams* params, uint8_t* out);
AliceFontParams aa_font_params_decode(const uint8_t* data);

float aa_font_params_stroke_half_width(const AliceFontParams* p);
float aa_font_params_thick_half_width(const AliceFontParams* p);
float aa_font_params_thin_half_width(const AliceFontParams* p);
float aa_font_params_serif_length(const AliceFontParams* p);

// --- GlyphGenerator ---

AliceFontGenerator aa_font_generator_new(const AliceFontParams* params);
void               aa_font_generator_free(AliceFontGenerator gen);
AliceFontGlyphSdf  aa_font_generator_generate(AliceFontGenerator gen, uint8_t ch);

// --- GlyphSdf ---

void    aa_font_glyph_free(AliceFontGlyphSdf sdf);
float   aa_font_glyph_sample(const AliceFontGlyphSdf sdf, float u, float v);
bool    aa_font_glyph_is_inside(const AliceFontGlyphSdf sdf, float u, float v);
float   aa_font_glyph_advance(const AliceFontGlyphSdf sdf);
const float* aa_font_glyph_data_ptr(const AliceFontGlyphSdf sdf);
uint32_t     aa_font_glyph_sdf_size();

// --- FontLicense ---

AliceFontLicense aa_font_license_parametric_free(const uint8_t* params_encoded);
AliceFontLicense aa_font_license_for_game(
    const uint8_t* params_encoded, uint32_t title_id, uint8_t platforms);
uint32_t aa_font_license_validate(
    const AliceFontLicense* lic, uint32_t current_epoch, uint8_t platform,
    uint16_t required_right, uint32_t title_id, uint16_t current_seats,
    const uint8_t* params_encoded);
bool aa_font_license_is_parametric_free(const AliceFontLicense* lic);

// --- GameTextStyle ---

AliceFontGameTextStyle aa_font_style_default();
AliceFontGameTextStyle aa_font_style_outlined();
AliceFontGameTextStyle aa_font_style_shadowed();
AliceFontGameTextStyle aa_font_style_neon();
void                   aa_font_style_free(AliceFontGameTextStyle style);
float                  aa_font_style_glyph(
    const AliceFontGlyphSdf sdf, const AliceFontGameTextStyle style, float* out_rgba);

} // extern "C"

// --- RAII wrappers (C++) ---

#ifdef __cplusplus

#include <memory>

namespace AliceFont
{

// Custom deleters for RAII
struct GeneratorDeleter { void operator()(void* p) const { aa_font_generator_free(p); } };
struct GlyphDeleter     { void operator()(void* p) const { aa_font_glyph_free(p); } };
struct StyleDeleter     { void operator()(void* p) const { aa_font_style_free(p); } };

using GeneratorPtr = std::unique_ptr<void, GeneratorDeleter>;
using GlyphPtr     = std::unique_ptr<void, GlyphDeleter>;
using StylePtr     = std::unique_ptr<void, StyleDeleter>;

inline GeneratorPtr CreateGenerator(const AliceFontParams& Params)
{
    return GeneratorPtr(aa_font_generator_new(&Params));
}

inline GlyphPtr GenerateGlyph(const GeneratorPtr& Gen, uint8_t Ch)
{
    return GlyphPtr(aa_font_generator_generate(Gen.get(), Ch));
}

inline StylePtr CreateDefaultStyle()  { return StylePtr(aa_font_style_default()); }
inline StylePtr CreateOutlinedStyle() { return StylePtr(aa_font_style_outlined()); }
inline StylePtr CreateShadowedStyle() { return StylePtr(aa_font_style_shadowed()); }
inline StylePtr CreateNeonStyle()     { return StylePtr(aa_font_style_neon()); }

} // namespace AliceFont

#endif // __cplusplus

// ALICE-Font Unity C# Bindings
// DllImport wrapper for alice_font FFI (C-ABI)
//
// Usage:
//   1. Build: cargo build --release --features ffi
//   2. Copy target/release/libalice_font.dylib (macOS) / alice_font.dll (Windows) to Assets/Plugins/
//   3. Use AliceFont.* from any C# script
//
// License: MIT
// Author: Moroya Sakamoto

using System;
using System.Runtime.InteropServices;

namespace AliceEngine.Font
{
    // MetaFontParams — 40-byte parametric font descriptor (repr(C), 10 × f32)
    [StructLayout(LayoutKind.Sequential)]
    public struct MetaFontParams
    {
        public float weight;
        public float width;
        public float serif;
        public float contrast;
        public float slant;
        public float xHeight;
        public float capHeight;
        public float ascender;
        public float descender;
        public float roundness;
    }

    // FontLicense — 32-byte wire format (repr(C))
    [StructLayout(LayoutKind.Sequential)]
    public struct FontLicense
    {
        public ulong contentHash;
        public ulong paramsHash;
        public uint titleId;
        public uint expiresEpoch;
        public ushort rights;
        public ushort maxSeats;
        public byte licenseType;
        public byte platforms;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 2)]
        public byte[] reserved;
    }

    public static class AliceFont
    {
#if UNITY_IOS && !UNITY_EDITOR
        private const string DLL = "__Internal";
#else
        private const string DLL = "alice_font";
#endif

        // --- MetaFontParams presets ---

        [DllImport(DLL)] public static extern MetaFontParams aa_font_params_sans_regular();
        [DllImport(DLL)] public static extern MetaFontParams aa_font_params_sans_bold();
        [DllImport(DLL)] public static extern MetaFontParams aa_font_params_serif_regular();
        [DllImport(DLL)] public static extern MetaFontParams aa_font_params_serif_italic();
        [DllImport(DLL)] public static extern MetaFontParams aa_font_params_mono_regular();
        [DllImport(DLL)] public static extern MetaFontParams aa_font_params_display_heavy();

        [DllImport(DLL)] public static extern MetaFontParams aa_font_params_lerp(
            ref MetaFontParams a, ref MetaFontParams b, float t);

        [DllImport(DLL)] public static extern unsafe void aa_font_params_encode(
            ref MetaFontParams p, byte* outBuf);

        [DllImport(DLL)] public static extern unsafe MetaFontParams aa_font_params_decode(byte* data);

        [DllImport(DLL)] public static extern float aa_font_params_stroke_half_width(ref MetaFontParams p);
        [DllImport(DLL)] public static extern float aa_font_params_thick_half_width(ref MetaFontParams p);
        [DllImport(DLL)] public static extern float aa_font_params_thin_half_width(ref MetaFontParams p);
        [DllImport(DLL)] public static extern float aa_font_params_serif_length(ref MetaFontParams p);

        // --- GlyphGenerator ---

        [DllImport(DLL)] public static extern IntPtr aa_font_generator_new(ref MetaFontParams p);
        [DllImport(DLL)] public static extern void aa_font_generator_free(IntPtr gen);
        [DllImport(DLL)] public static extern IntPtr aa_font_generator_generate(IntPtr gen, byte ch);

        // --- GlyphSdf ---

        [DllImport(DLL)] public static extern void aa_font_glyph_free(IntPtr sdf);
        [DllImport(DLL)] public static extern float aa_font_glyph_sample(IntPtr sdf, float u, float v);
        [DllImport(DLL)] public static extern bool aa_font_glyph_is_inside(IntPtr sdf, float u, float v);
        [DllImport(DLL)] public static extern float aa_font_glyph_advance(IntPtr sdf);
        [DllImport(DLL)] public static extern IntPtr aa_font_glyph_data_ptr(IntPtr sdf);
        [DllImport(DLL)] public static extern uint aa_font_glyph_sdf_size();

        // --- FontLicense ---

        [DllImport(DLL)] public static extern unsafe FontLicense aa_font_license_parametric_free(byte* paramsEncoded);
        [DllImport(DLL)] public static extern unsafe FontLicense aa_font_license_for_game(
            byte* paramsEncoded, uint titleId, byte platforms);
        [DllImport(DLL)] public static extern unsafe uint aa_font_license_validate(
            ref FontLicense lic, uint currentEpoch, byte platform,
            ushort requiredRight, uint titleId, ushort currentSeats, byte* paramsEncoded);
        [DllImport(DLL)] public static extern bool aa_font_license_is_parametric_free(ref FontLicense lic);

        // --- GameTextStyle ---

        [DllImport(DLL)] public static extern IntPtr aa_font_style_default();
        [DllImport(DLL)] public static extern IntPtr aa_font_style_outlined();
        [DllImport(DLL)] public static extern IntPtr aa_font_style_shadowed();
        [DllImport(DLL)] public static extern IntPtr aa_font_style_neon();
        [DllImport(DLL)] public static extern void aa_font_style_free(IntPtr style);
        [DllImport(DLL)] public static extern unsafe float aa_font_style_glyph(
            IntPtr sdf, IntPtr style, float* outRgba);

        // --- RAII wrappers ---

        public class Generator : IDisposable
        {
            private IntPtr _ptr;

            public Generator(MetaFontParams p)
            {
                _ptr = aa_font_generator_new(ref p);
            }

            public Glyph Generate(byte ch)
            {
                return new Glyph(aa_font_generator_generate(_ptr, ch));
            }

            public void Dispose()
            {
                if (_ptr != IntPtr.Zero)
                {
                    aa_font_generator_free(_ptr);
                    _ptr = IntPtr.Zero;
                }
            }
        }

        public class Glyph : IDisposable
        {
            private IntPtr _ptr;

            internal Glyph(IntPtr ptr) { _ptr = ptr; }

            public float Sample(float u, float v) => aa_font_glyph_sample(_ptr, u, v);
            public bool IsInside(float u, float v) => aa_font_glyph_is_inside(_ptr, u, v);
            public float Advance => aa_font_glyph_advance(_ptr);
            public IntPtr Ptr => _ptr;

            public void Dispose()
            {
                if (_ptr != IntPtr.Zero)
                {
                    aa_font_glyph_free(_ptr);
                    _ptr = IntPtr.Zero;
                }
            }
        }

        public class Style : IDisposable
        {
            private IntPtr _ptr;

            public static Style Default() => new Style(aa_font_style_default());
            public static Style Outlined() => new Style(aa_font_style_outlined());
            public static Style Shadowed() => new Style(aa_font_style_shadowed());
            public static Style Neon() => new Style(aa_font_style_neon());

            private Style(IntPtr ptr) { _ptr = ptr; }
            public IntPtr Ptr => _ptr;

            public void Dispose()
            {
                if (_ptr != IntPtr.Zero)
                {
                    aa_font_style_free(_ptr);
                    _ptr = IntPtr.Zero;
                }
            }
        }
    }
}

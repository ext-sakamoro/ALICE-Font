//! PyO3 Python bindings for ALICE-Font
//!
//! Provides Python-friendly wrappers for parametric font generation.
//! Enabled via `--features pyo3`.
//!
//! License: MIT
//! Author: Moroya Sakamoto

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::game::{style_glyph, GameTextStyle};
use crate::glyph::{GlyphGenerator, GlyphSdf, GLYPH_SDF_SIZE};
use crate::license::{FontLicense, LicenseValidator, PlatformRestriction};
use crate::param::MetaFontParams;

/// Python wrapper for MetaFontParams
#[pyclass(name = "MetaFontParams")]
#[derive(Clone)]
pub struct PyMetaFontParams {
    pub inner: MetaFontParams,
}

#[pymethods]
impl PyMetaFontParams {
    /// Create from 10 parameters
    #[new]
    #[pyo3(signature = (weight=0.45, width=1.0, serif=0.0, contrast=0.15, slant=0.0, x_height=0.52, cap_height=0.72, ascender=0.80, descender=0.22, roundness=0.3))]
    fn new(
        weight: f32,
        width: f32,
        serif: f32,
        contrast: f32,
        slant: f32,
        x_height: f32,
        cap_height: f32,
        ascender: f32,
        descender: f32,
        roundness: f32,
    ) -> Self {
        Self {
            inner: MetaFontParams {
                weight,
                width,
                serif,
                contrast,
                slant,
                x_height,
                cap_height,
                ascender,
                descender,
                roundness,
            },
        }
    }

    /// Sans-serif regular preset
    #[staticmethod]
    fn sans_regular() -> Self {
        Self {
            inner: MetaFontParams::sans_regular(),
        }
    }

    /// Sans-serif bold preset
    #[staticmethod]
    fn sans_bold() -> Self {
        Self {
            inner: MetaFontParams::sans_bold(),
        }
    }

    /// Serif regular preset
    #[staticmethod]
    fn serif_regular() -> Self {
        Self {
            inner: MetaFontParams::serif_regular(),
        }
    }

    /// Serif italic preset
    #[staticmethod]
    fn serif_italic() -> Self {
        Self {
            inner: MetaFontParams::serif_italic(),
        }
    }

    /// Monospace regular preset
    #[staticmethod]
    fn mono_regular() -> Self {
        Self {
            inner: MetaFontParams::mono_regular(),
        }
    }

    /// Display heavy preset
    #[staticmethod]
    fn display_heavy() -> Self {
        Self {
            inner: MetaFontParams::display_heavy(),
        }
    }

    /// Interpolate between two parameter sets
    fn lerp(&self, other: &PyMetaFontParams, t: f32) -> Self {
        Self {
            inner: self.inner.lerp(&other.inner, t),
        }
    }

    /// Encode to 40-byte wire format
    fn encode(&self) -> Vec<u8> {
        self.inner.encode().to_vec()
    }

    /// Decode from 40-byte wire format
    #[staticmethod]
    fn decode(data: Vec<u8>) -> PyResult<Self> {
        if data.len() != 40 {
            return Err(PyValueError::new_err("data must be exactly 40 bytes"));
        }
        let mut buf = [0u8; 40];
        buf.copy_from_slice(&data);
        Ok(Self {
            inner: MetaFontParams::decode(&buf),
        })
    }

    /// Stroke half-width (em units)
    fn stroke_half_width(&self) -> f32 {
        self.inner.stroke_half_width()
    }

    #[getter]
    fn weight(&self) -> f32 {
        self.inner.weight
    }
    #[getter]
    fn width(&self) -> f32 {
        self.inner.width
    }
    #[getter]
    fn serif(&self) -> f32 {
        self.inner.serif
    }
    #[getter]
    fn contrast(&self) -> f32 {
        self.inner.contrast
    }
    #[getter]
    fn slant(&self) -> f32 {
        self.inner.slant
    }

    fn __repr__(&self) -> String {
        alloc::format!(
            "MetaFontParams(weight={:.3}, width={:.3}, serif={:.3}, contrast={:.3}, slant={:.3})",
            self.inner.weight,
            self.inner.width,
            self.inner.serif,
            self.inner.contrast,
            self.inner.slant
        )
    }
}

/// Python wrapper for GlyphGenerator
#[pyclass(name = "GlyphGenerator")]
pub struct PyGlyphGenerator {
    inner: GlyphGenerator,
}

#[pymethods]
impl PyGlyphGenerator {
    #[new]
    fn new(params: &PyMetaFontParams) -> Self {
        Self {
            inner: GlyphGenerator::new(&params.inner),
        }
    }

    /// Generate SDF for a character (ASCII byte value)
    fn generate(&self, ch: u8) -> PyGlyphSdf {
        PyGlyphSdf {
            inner: self.inner.generate(ch),
        }
    }

    /// Generate SDF for a character string (first char only)
    fn generate_char(&self, ch: &str) -> PyResult<PyGlyphSdf> {
        let byte = ch
            .as_bytes()
            .first()
            .ok_or_else(|| PyValueError::new_err("empty string"))?;
        Ok(PyGlyphSdf {
            inner: self.inner.generate(*byte),
        })
    }
}

/// Python wrapper for GlyphSdf
#[pyclass(name = "GlyphSdf")]
pub struct PyGlyphSdf {
    inner: GlyphSdf,
}

#[pymethods]
impl PyGlyphSdf {
    /// Sample SDF at normalized coordinates (0..1, 0..1)
    fn sample(&self, u: f32, v: f32) -> f32 {
        self.inner.sample(u, v)
    }

    /// Check if point is inside glyph
    fn is_inside(&self, u: f32, v: f32) -> bool {
        self.inner.is_inside(u, v)
    }

    /// Advance width (em units)
    #[getter]
    fn advance(&self) -> f32 {
        self.inner.advance
    }

    /// Get SDF data as flat list of f32 (32*32 = 1024 values)
    fn data(&self) -> Vec<f32> {
        self.inner.data.to_vec()
    }

    /// SDF grid resolution
    #[getter]
    fn size(&self) -> usize {
        GLYPH_SDF_SIZE
    }
}

/// Python wrapper for FontLicense
#[pyclass(name = "FontLicense")]
#[derive(Clone)]
pub struct PyFontLicense {
    inner: FontLicense,
}

#[pymethods]
impl PyFontLicense {
    /// Create a free parametric font license
    #[staticmethod]
    fn parametric_free(params: &PyMetaFontParams) -> Self {
        let encoded = params.inner.encode();
        Self {
            inner: FontLicense::parametric_free(&encoded),
        }
    }

    /// Create a game-title license
    #[staticmethod]
    fn for_game_title(params: &PyMetaFontParams, title_id: u32, platforms: u8) -> Self {
        let encoded = params.inner.encode();
        Self {
            inner: FontLicense::for_game_title(&encoded, title_id, PlatformRestriction(platforms)),
        }
    }

    /// Check if this is a free parametric license
    fn is_parametric_free(&self) -> bool {
        LicenseValidator::is_parametric_free(&self.inner)
    }

    /// Encode to 32-byte wire format
    fn encode(&self) -> Vec<u8> {
        self.inner.encode().to_vec()
    }
}

/// Style a glyph with game text effects. Returns RGBA data as flat f32 list.
#[pyfunction]
fn style_glyph_default(sdf: &PyGlyphSdf) -> Vec<f32> {
    let style = GameTextStyle::default_style();
    let styled = style_glyph(&sdf.inner, &style);
    let mut out = alloc::vec::Vec::with_capacity(GLYPH_SDF_SIZE * GLYPH_SDF_SIZE * 4);
    for p in &styled.pixels {
        out.push(p.r);
        out.push(p.g);
        out.push(p.b);
        out.push(p.a);
    }
    out
}

/// Style a glyph with neon effect. Returns RGBA data as flat f32 list.
#[pyfunction]
fn style_glyph_neon(sdf: &PyGlyphSdf) -> Vec<f32> {
    let style = GameTextStyle::neon();
    let styled = style_glyph(&sdf.inner, &style);
    let mut out = alloc::vec::Vec::with_capacity(GLYPH_SDF_SIZE * GLYPH_SDF_SIZE * 4);
    for p in &styled.pixels {
        out.push(p.r);
        out.push(p.g);
        out.push(p.b);
        out.push(p.a);
    }
    out
}

/// Get the SDF glyph tile size
#[pyfunction]
fn glyph_sdf_size() -> usize {
    GLYPH_SDF_SIZE
}

/// Register the alice_font Python module
pub fn register_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMetaFontParams>()?;
    m.add_class::<PyGlyphGenerator>()?;
    m.add_class::<PyGlyphSdf>()?;
    m.add_class::<PyFontLicense>()?;
    m.add_function(wrap_pyfunction!(style_glyph_default, m)?)?;
    m.add_function(wrap_pyfunction!(style_glyph_neon, m)?)?;
    m.add_function(wrap_pyfunction!(glyph_sdf_size, m)?)?;
    Ok(())
}

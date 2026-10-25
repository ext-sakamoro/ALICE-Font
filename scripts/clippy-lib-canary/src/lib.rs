//! Each function uses one form that .github/clippy-lib/clippy.toml forbids.
//! scripts/clippy_lib_gate.py runs clippy with that configuration and requires a
//! disallowed_methods diagnostic naming each form's path.

pub fn f32_sin(a: f32) -> f32 { a.sin() }
pub fn f32_cos(a: f32) -> f32 { a.cos() }
pub fn f32_sin_cos(a: f32) -> (f32, f32) { a.sin_cos() }
pub fn f32_tan(a: f32) -> f32 { a.tan() }
pub fn f32_asin(a: f32) -> f32 { a.asin() }
pub fn f32_acos(a: f32) -> f32 { a.acos() }
pub fn f32_atan(a: f32) -> f32 { a.atan() }
pub fn f32_atan2(a: f32, b: f32) -> f32 { a.atan2(b) }
pub fn f32_sinh(a: f32) -> f32 { a.sinh() }
pub fn f32_cosh(a: f32) -> f32 { a.cosh() }
pub fn f32_tanh(a: f32) -> f32 { a.tanh() }
pub fn f32_asinh(a: f32) -> f32 { a.asinh() }
pub fn f32_acosh(a: f32) -> f32 { a.acosh() }
pub fn f32_atanh(a: f32) -> f32 { a.atanh() }
pub fn f32_exp(a: f32) -> f32 { a.exp() }
pub fn f32_exp2(a: f32) -> f32 { a.exp2() }
pub fn f32_exp_m1(a: f32) -> f32 { a.exp_m1() }
pub fn f32_ln(a: f32) -> f32 { a.ln() }
pub fn f32_ln_1p(a: f32) -> f32 { a.ln_1p() }
pub fn f32_log(a: f32, b: f32) -> f32 { a.log(b) }
pub fn f32_log2(a: f32) -> f32 { a.log2() }
pub fn f32_log10(a: f32) -> f32 { a.log10() }
pub fn f32_powf(a: f32, b: f32) -> f32 { a.powf(b) }
pub fn f32_cbrt(a: f32) -> f32 { a.cbrt() }
pub fn f32_hypot(a: f32, b: f32) -> f32 { a.hypot(b) }
pub fn f32_powi(a: f32) -> f32 { a.powi(2) }
pub fn f64_sin(d: f64) -> f64 { d.sin() }
pub fn f64_cos(d: f64) -> f64 { d.cos() }
pub fn f64_sin_cos(d: f64) -> (f64, f64) { d.sin_cos() }
pub fn f64_tan(d: f64) -> f64 { d.tan() }
pub fn f64_asin(d: f64) -> f64 { d.asin() }
pub fn f64_acos(d: f64) -> f64 { d.acos() }
pub fn f64_atan(d: f64) -> f64 { d.atan() }
pub fn f64_atan2(d: f64, e: f64) -> f64 { d.atan2(e) }
pub fn f64_sinh(d: f64) -> f64 { d.sinh() }
pub fn f64_cosh(d: f64) -> f64 { d.cosh() }
pub fn f64_tanh(d: f64) -> f64 { d.tanh() }
pub fn f64_asinh(d: f64) -> f64 { d.asinh() }
pub fn f64_acosh(d: f64) -> f64 { d.acosh() }
pub fn f64_atanh(d: f64) -> f64 { d.atanh() }
pub fn f64_exp(d: f64) -> f64 { d.exp() }
pub fn f64_exp2(d: f64) -> f64 { d.exp2() }
pub fn f64_exp_m1(d: f64) -> f64 { d.exp_m1() }
pub fn f64_ln(d: f64) -> f64 { d.ln() }
pub fn f64_ln_1p(d: f64) -> f64 { d.ln_1p() }
pub fn f64_log(d: f64, e: f64) -> f64 { d.log(e) }
pub fn f64_log2(d: f64) -> f64 { d.log2() }
pub fn f64_log10(d: f64) -> f64 { d.log10() }
pub fn f64_powf(d: f64, e: f64) -> f64 { d.powf(e) }
pub fn f64_cbrt(d: f64) -> f64 { d.cbrt() }
pub fn f64_hypot(d: f64, e: f64) -> f64 { d.hypot(e) }
pub fn f64_powi(d: f64) -> f64 { d.powi(2) }
pub fn clean_product(a: f32, b: f32) -> f32 { a * b }

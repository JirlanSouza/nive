//! Primitive radius tokens in logical pixels.
//!
//! `FULL` is intentionally much larger than normal UI bounds so renderers clamp
//! it into pill/circle corners.

/// Extra-small corner radius.
pub const XS: f32 = 2.0;
/// Small corner radius.
pub const SM: f32 = 4.0;
/// Medium corner radius.
pub const MD: f32 = 6.0;
/// Large corner radius.
pub const LG: f32 = 8.0;
/// Extra-large corner radius.
pub const XL: f32 = 12.0;
/// Extra-extra-large corner radius.
pub const XXL: f32 = 16.0;
/// Internal large numeric radius escape hatch.
pub const XXXL: f32 = 24.0;
/// Internal largest numeric radius escape hatch.
pub const XXXXL: f32 = 32.0;
/// Pill/full radius token.
pub const FULL: f32 = 9999.0;

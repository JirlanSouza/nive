//! Bundled default typography assets.
//!
//! `nive-ui` bundles Inter (Regular, SemiBold) and Geist Mono (Regular,
//! Medium) as static font assets behind the default-on `bundled-fonts` cargo
//! feature, so the default appearance does not depend on fonts installed on
//! the host operating system. Geist Mono Regular supports code/content
//! roles; Geist Mono Medium is bundled for the confirmed 11px
//! technical-metadata style, whose consuming widget lands in a later change.
//!
//! `nive-runtime` registers [`bundled()`] automatically and defaults the
//! application font to [`default_font()`] unless the app overrides it.
//! Consumers using `nive-ui` directly with Iced (without `nive-runtime`) can
//! feed [`bundled()`] into their own font registration.
//!
//! A custom theme may replace the default families; when it does, the
//! application is responsible for registering the replacement font data
//! itself (`bundled()`/`default_font()` only ever describe the framework
//! defaults).
//!
//! Disable the `bundled-fonts` feature (`default-features = false`) to
//! exclude the embedded bytes from the binary; [`bundled()`] then returns an
//! empty slice and [`default_font()`] falls back to [`Font::DEFAULT`].

use iced::Font;

use crate::tokens::typography::UI;

#[cfg(feature = "bundled-fonts")]
const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/inter/Inter-Regular.ttf");
#[cfg(feature = "bundled-fonts")]
const INTER_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/inter/Inter-SemiBold.ttf");
#[cfg(feature = "bundled-fonts")]
const GEIST_MONO_REGULAR: &[u8] =
    include_bytes!("../assets/fonts/geist-mono/GeistMono-Regular.ttf");
#[cfg(feature = "bundled-fonts")]
const GEIST_MONO_MEDIUM: &[u8] = include_bytes!("../assets/fonts/geist-mono/GeistMono-Medium.ttf");

/// Returns the raw bytes of every bundled font face: Inter Regular, Inter
/// SemiBold, Geist Mono Regular, and Geist Mono Medium, in that order.
///
/// Empty when the `bundled-fonts` feature is disabled.
pub fn bundled() -> &'static [&'static [u8]] {
    #[cfg(feature = "bundled-fonts")]
    {
        &[
            INTER_REGULAR,
            INTER_SEMIBOLD,
            GEIST_MONO_REGULAR,
            GEIST_MONO_MEDIUM,
        ]
    }
    #[cfg(not(feature = "bundled-fonts"))]
    {
        &[]
    }
}

/// Returns the default application font (Inter), independent of the host OS.
///
/// Falls back to [`Font::DEFAULT`] when the `bundled-fonts` feature is
/// disabled, since no bundled Inter data is embedded in that configuration.
pub fn default_font() -> Font {
    #[cfg(feature = "bundled-fonts")]
    {
        UI.normal()
    }
    #[cfg(not(feature = "bundled-fonts"))]
    {
        Font::DEFAULT
    }
}

#[cfg(test)]
mod fonts_tests {
    use super::*;

    #[test]
    #[cfg(feature = "bundled-fonts")]
    fn bundled_exposes_all_four_confirmed_faces() {
        assert_eq!(bundled().len(), 4);
        assert!(bundled().iter().all(|face| !face.is_empty()));
    }

    #[test]
    #[cfg(feature = "bundled-fonts")]
    fn default_font_is_inter() {
        assert_eq!(default_font(), UI.normal());
    }

    #[test]
    #[cfg(not(feature = "bundled-fonts"))]
    fn bundled_is_empty_without_the_feature() {
        assert!(bundled().is_empty());
    }

    #[test]
    #[cfg(not(feature = "bundled-fonts"))]
    fn default_font_falls_back_to_iced_default_without_the_feature() {
        assert_eq!(default_font(), Font::DEFAULT);
    }
}

use iced::border::Radius;

use crate::tokens::radius as token_radius;

/// Ordered corner-radius scale used by theme shape resolution.
///
/// `Full` carries pill/circle semantics and resolves to
/// [`crate::tokens::radius::FULL`], not the largest numeric token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeSize {
    /// Square corners.
    None,
    /// Extra-small radius.
    Xs,
    /// Small radius.
    Sm,
    /// Medium radius.
    Md,
    /// Large radius.
    Lg,
    /// Extra-large radius.
    Xl,
    /// Extra-extra-large radius.
    Xxl,
    /// Pill/circle radius.
    Full,
}

/// Concrete shape resolved from a [`ShapeSize`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeSpec {
    pub radius: Radius,
}

/// Theme-owned shape scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeScale {
    pub none: ShapeSpec,
    pub xs: ShapeSpec,
    pub sm: ShapeSpec,
    pub md: ShapeSpec,
    pub lg: ShapeSpec,
    pub xl: ShapeSpec,
    pub xxl: ShapeSpec,
    pub full: ShapeSpec,
}

/// Default shape scale.
pub const SCALE: ShapeScale = ShapeScale {
    none: ShapeSpec::new(0.0),
    xs: ShapeSpec::new(token_radius::XS),
    sm: ShapeSpec::new(token_radius::SM),
    md: ShapeSpec::new(token_radius::MD),
    lg: ShapeSpec::new(token_radius::LG),
    xl: ShapeSpec::new(token_radius::XL),
    xxl: ShapeSpec::new(token_radius::XXL),
    full: ShapeSpec::new(token_radius::FULL),
};

/// Returns the default shape scale.
pub const fn scale() -> ShapeScale {
    SCALE
}

/// Resolves a shape size to its scalar radius value.
pub const fn radius(size: ShapeSize) -> f32 {
    match size {
        ShapeSize::None => 0.0,
        ShapeSize::Xs => token_radius::XS,
        ShapeSize::Sm => token_radius::SM,
        ShapeSize::Md => token_radius::MD,
        ShapeSize::Lg => token_radius::LG,
        ShapeSize::Xl => token_radius::XL,
        ShapeSize::Xxl => token_radius::XXL,
        ShapeSize::Full => token_radius::FULL,
    }
}

impl ShapeScale {
    /// Returns the concrete shape for a size.
    pub fn get(self, size: ShapeSize) -> ShapeSpec {
        match size {
            ShapeSize::None => self.none,
            ShapeSize::Xs => self.xs,
            ShapeSize::Sm => self.sm,
            ShapeSize::Md => self.md,
            ShapeSize::Lg => self.lg,
            ShapeSize::Xl => self.xl,
            ShapeSize::Xxl => self.xxl,
            ShapeSize::Full => self.full,
        }
    }
}

impl ShapeSpec {
    /// Creates a uniform-radius shape spec.
    pub const fn new(radius: f32) -> Self {
        let radius = Radius {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        };

        Self { radius }
    }

    /// Returns the Iced radius.
    pub const fn radius(self) -> Radius {
        self.radius
    }

    /// Returns the scalar top-left radius value.
    pub const fn radius_value(self) -> f32 {
        self.radius.top_left
    }
}

#[cfg(test)]
mod shape_tests {
    use super::*;

    #[test]
    fn shape_sizes_map_to_radius_tokens() {
        assert_eq!(radius(ShapeSize::None), 0.0);
        assert_eq!(radius(ShapeSize::Xs), token_radius::XS);
        assert_eq!(radius(ShapeSize::Sm), token_radius::SM);
        assert_eq!(radius(ShapeSize::Md), token_radius::MD);
        assert_eq!(radius(ShapeSize::Lg), token_radius::LG);
        assert_eq!(radius(ShapeSize::Xl), token_radius::XL);
        assert_eq!(radius(ShapeSize::Xxl), token_radius::XXL);
        assert_eq!(radius(ShapeSize::Full), token_radius::FULL);
    }

    #[test]
    fn radius_tokens_are_decided_integer_values() {
        assert_eq!(token_radius::XS, 2.0);
        assert_eq!(token_radius::SM, 4.0);
        assert_eq!(token_radius::MD, 6.0);
        assert_eq!(token_radius::LG, 8.0);
        assert_eq!(token_radius::XL, 12.0);
        assert_eq!(token_radius::XXL, 16.0);
        assert_eq!(token_radius::XXXL, 24.0);
        assert_eq!(token_radius::XXXXL, 32.0);
        assert_eq!(token_radius::FULL, 9999.0);
    }

    #[test]
    fn shape_scale_is_ordered_by_radius() {
        let scale = scale();

        assert!(
            scale.get(ShapeSize::None).radius_value() < scale.get(ShapeSize::Xs).radius_value()
        );
        assert!(scale.get(ShapeSize::Xs).radius_value() < scale.get(ShapeSize::Sm).radius_value());
        assert!(scale.get(ShapeSize::Sm).radius_value() < scale.get(ShapeSize::Md).radius_value());
        assert!(scale.get(ShapeSize::Md).radius_value() < scale.get(ShapeSize::Lg).radius_value());
        assert!(scale.get(ShapeSize::Lg).radius_value() < scale.get(ShapeSize::Xl).radius_value());
        assert!(scale.get(ShapeSize::Xl).radius_value() < scale.get(ShapeSize::Xxl).radius_value());
        assert!(
            scale.get(ShapeSize::Xxl).radius_value() < scale.get(ShapeSize::Full).radius_value()
        );
    }

    #[test]
    fn full_shape_uses_pill_token() {
        assert_eq!(
            scale().get(ShapeSize::Full).radius_value(),
            token_radius::FULL
        );
    }
}

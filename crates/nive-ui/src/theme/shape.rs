use crate::tokens::radius as token_radius;

use iced::border::Radius;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeRole {
    None,
    ExtraSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeSpec {
    pub radius: Radius,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeScale {
    pub none: ShapeSpec,
    pub extra_small: ShapeSpec,
    pub small: ShapeSpec,
    pub medium: ShapeSpec,
    pub large: ShapeSpec,
    pub extra_large: ShapeSpec,
    pub full: ShapeSpec,
}

pub const SCALE: ShapeScale = ShapeScale {
    none: ShapeSpec::new(0.0),
    extra_small: ShapeSpec::new(token_radius::XS),
    small: ShapeSpec::new(token_radius::SM),
    medium: ShapeSpec::new(token_radius::MD),
    large: ShapeSpec::new(token_radius::LG),
    extra_large: ShapeSpec::new(token_radius::XL),
    full: ShapeSpec::new(token_radius::XXXXL),
};

pub const fn scale() -> ShapeScale {
    SCALE
}

pub const fn radius(role: ShapeRole) -> f32 {
    match role {
        ShapeRole::None => 0.0,
        ShapeRole::ExtraSmall => token_radius::XS,
        ShapeRole::Small => token_radius::SM,
        ShapeRole::Medium => token_radius::MD,
        ShapeRole::Large => token_radius::LG,
        ShapeRole::ExtraLarge => token_radius::XL,
        ShapeRole::Full => token_radius::XXXXL,
    }
}

impl ShapeScale {
    pub fn get(self, role: ShapeRole) -> ShapeSpec {
        match role {
            ShapeRole::None => self.none,
            ShapeRole::ExtraSmall => self.extra_small,
            ShapeRole::Small => self.small,
            ShapeRole::Medium => self.medium,
            ShapeRole::Large => self.large,
            ShapeRole::ExtraLarge => self.extra_large,
            ShapeRole::Full => self.full,
        }
    }
}

impl ShapeSpec {
    pub const fn new(radius: f32) -> Self {
        let radius = Radius {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        };

        Self { radius }
    }

    pub const fn radius(self) -> Radius {
        self.radius
    }

    pub const fn radius_value(self) -> f32 {
        self.radius.top_left
    }
}

#[cfg(test)]
mod shape_tests {
    use super::*;

    #[test]
    fn shape_roles_map_to_existing_radius_scale() {
        assert_eq!(radius(ShapeRole::None), 0.0);
        assert_eq!(radius(ShapeRole::ExtraSmall), token_radius::XS);
        assert_eq!(radius(ShapeRole::Small), token_radius::SM);
        assert_eq!(radius(ShapeRole::Medium), token_radius::MD);
        assert_eq!(radius(ShapeRole::Large), token_radius::LG);
        assert_eq!(radius(ShapeRole::ExtraLarge), token_radius::XL);
        assert_eq!(radius(ShapeRole::Full), token_radius::XXXXL);
    }

    #[test]
    fn shape_scale_is_ordered_by_radius() {
        let scale = scale();

        assert!(
            scale.get(ShapeRole::None).radius_value()
                < scale.get(ShapeRole::ExtraSmall).radius_value()
        );
        assert!(
            scale.get(ShapeRole::ExtraSmall).radius_value()
                < scale.get(ShapeRole::Small).radius_value()
        );
        assert!(
            scale.get(ShapeRole::Small).radius_value()
                < scale.get(ShapeRole::Medium).radius_value()
        );
        assert!(
            scale.get(ShapeRole::Medium).radius_value()
                < scale.get(ShapeRole::Large).radius_value()
        );
        assert!(
            scale.get(ShapeRole::Large).radius_value()
                < scale.get(ShapeRole::ExtraLarge).radius_value()
        );
        assert!(
            scale.get(ShapeRole::ExtraLarge).radius_value()
                < scale.get(ShapeRole::Full).radius_value()
        );
    }
}

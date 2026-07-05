use crate::tokens::spacing as token_spacing;

use iced::Padding;

use super::density::ThemeDensity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceStep {
    None,
    Xxs,
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
    Xxxl,
    Page,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapRole {
    Tight,
    Related,
    Content,
    Section,
    Page,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddingRole {
    None,
    Compact,
    Control,
    Content,
    Panel,
    Dialog,
    Page,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacingScale {
    pub none: f32,
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub xxxl: f32,
    pub page: f32,
}

pub const SCALE: SpacingScale = SpacingScale {
    none: token_spacing::SPACE_0,
    xxs: token_spacing::SPACE_0_5,
    xs: token_spacing::SPACE_1,
    sm: token_spacing::SPACE_1_5,
    md: token_spacing::SPACE_2,
    lg: token_spacing::SPACE_3,
    xl: token_spacing::SPACE_4,
    xxl: token_spacing::SPACE_6,
    xxxl: token_spacing::SPACE_8,
    page: token_spacing::SPACE_12,
};

pub const fn scale() -> SpacingScale {
    SCALE
}

pub const fn scale_for_density(density: ThemeDensity) -> SpacingScale {
    match density {
        ThemeDensity::Comfortable => SpacingScale {
            none: 0.0,
            xxs: 3.0,
            xs: 6.0,
            sm: 8.0,
            md: 10.0,
            lg: 14.0,
            xl: 20.0,
            xxl: 28.0,
            xxxl: 40.0,
            page: 56.0,
        },
        ThemeDensity::Standard => SCALE,
        ThemeDensity::Compact => SpacingScale {
            none: 0.0,
            xxs: 1.0,
            xs: 3.0,
            sm: 4.0,
            md: 6.0,
            lg: 8.0,
            xl: 12.0,
            xxl: 16.0,
            xxxl: 24.0,
            page: 32.0,
        },
    }
}

impl SpacingScale {
    pub fn step(self, step: SpaceStep) -> f32 {
        self.space(step)
    }

    pub fn space(self, step: SpaceStep) -> f32 {
        match step {
            SpaceStep::None => self.none,
            SpaceStep::Xxs => self.xxs,
            SpaceStep::Xs => self.xs,
            SpaceStep::Sm => self.sm,
            SpaceStep::Md => self.md,
            SpaceStep::Lg => self.lg,
            SpaceStep::Xl => self.xl,
            SpaceStep::Xxl => self.xxl,
            SpaceStep::Xxxl => self.xxxl,
            SpaceStep::Page => self.page,
        }
    }

    pub fn gap(self, role: GapRole) -> f32 {
        match role {
            GapRole::Tight => self.xs,
            GapRole::Related => self.md,
            GapRole::Content => self.lg,
            GapRole::Section => self.xl,
            GapRole::Page => self.xxxl,
        }
    }

    pub fn padding(self, role: PaddingRole) -> Padding {
        match role {
            PaddingRole::None => Padding::ZERO,
            PaddingRole::Compact => Padding::new(self.md),
            PaddingRole::Control => Padding::ZERO.vertical(self.xs).horizontal(self.sm),
            PaddingRole::Content => Padding::new(self.lg),
            PaddingRole::Panel => Padding::new(self.xl),
            PaddingRole::Dialog => Padding::new(self.xxl),
            PaddingRole::Page => Padding::new(self.page),
        }
    }
}

#[cfg(test)]
mod spacing_tests {
    use super::*;

    #[test]
    fn semantic_gaps_are_ordered_by_visual_distance() {
        let scale = scale();

        assert!(scale.gap(GapRole::Tight) < scale.gap(GapRole::Related));
        assert!(scale.gap(GapRole::Related) < scale.gap(GapRole::Content));
        assert!(scale.gap(GapRole::Content) < scale.gap(GapRole::Section));
        assert!(scale.gap(GapRole::Section) < scale.gap(GapRole::Page));
    }

    #[test]
    fn control_padding_is_horizontal_first() {
        let padding = scale().padding(PaddingRole::Control);

        assert!(padding.left > padding.top);
        assert_eq!(padding.left, padding.right);
        assert_eq!(padding.top, padding.bottom);
    }

    #[test]
    fn space_step_none_is_zero_for_all_densities() {
        for density in ThemeDensity::ALL {
            let spacing = scale_for_density(density);
            assert_eq!(spacing.space(SpaceStep::None), 0.0);
        }
    }

    #[test]
    fn density_spacing_is_ordered_compact_le_standard_le_comfortable() {
        let compact = scale_for_density(ThemeDensity::Compact);
        let standard = scale_for_density(ThemeDensity::Standard);
        let comfortable = scale_for_density(ThemeDensity::Comfortable);

        for step in [
            SpaceStep::Xxs,
            SpaceStep::Xs,
            SpaceStep::Sm,
            SpaceStep::Md,
            SpaceStep::Lg,
            SpaceStep::Xl,
            SpaceStep::Xxl,
            SpaceStep::Xxxl,
            SpaceStep::Page,
        ] {
            assert!(
                compact.space(step) <= standard.space(step),
                "compact {:?} {} > standard {}",
                step,
                compact.space(step),
                standard.space(step)
            );
            assert!(
                standard.space(step) <= comfortable.space(step),
                "standard {} > comfortable {:?} {}",
                standard.space(step),
                step,
                comfortable.space(step)
            );
        }
    }
}

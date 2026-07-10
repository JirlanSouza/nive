use iced::Padding;

use super::density::ThemeDensity;
use super::shape::{ShapeScale, ShapeSize, ShapeSpec};
use super::spacing::{SpaceStep, SpacingScale};
use super::typography::{TextStyle, TypographyRole, TypographyScale};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSize {
    Xs,
    Sm,
    Md,
    Lg,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlMetrics {
    pub height: f32,
    pub shape: ShapeSpec,
    pub text_style: TextStyle,
    pub padding: Padding,
    pub radius: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlMetricsScale {
    pub xs: ControlMetrics,
    pub sm: ControlMetrics,
    pub md: ControlMetrics,
    pub lg: ControlMetrics,
}

pub fn control_metrics(size: ControlSize) -> ControlMetrics {
    scale(
        super::shape::scale(),
        super::typography::scale(),
        super::spacing::scale(),
    )
    .get(size)
}

pub fn scale(
    shapes: ShapeScale,
    typography: TypographyScale,
    spacing: SpacingScale,
) -> ControlMetricsScale {
    scale_for_density(ThemeDensity::Standard, shapes, typography, spacing)
}

pub fn scale_for_density(
    density: ThemeDensity,
    shapes: ShapeScale,
    typography: TypographyScale,
    spacing: SpacingScale,
) -> ControlMetricsScale {
    let (xs_height, sm_height, md_height, lg_height) = match density {
        ThemeDensity::Comfortable => (28.0, 32.0, 36.0, 40.0),
        ThemeDensity::Standard => (24.0, 28.0, 32.0, 36.0),
        ThemeDensity::Compact => (20.0, 24.0, 28.0, 32.0),
    };

    let (xs_icon, sm_icon, md_icon, lg_icon) = match density {
        ThemeDensity::Comfortable => (14.0, 16.0, 16.0, 18.0),
        ThemeDensity::Standard => (12.0, 14.0, 14.0, 16.0),
        ThemeDensity::Compact => (10.0, 12.0, 14.0, 14.0),
    };

    ControlMetricsScale {
        xs: metrics(
            xs_height,
            shapes.get(ShapeSize::Sm),
            typography.get(TypographyRole::BodySmall),
            Padding::ZERO
                .vertical(spacing.step(SpaceStep::Xxs))
                .horizontal(spacing.step(SpaceStep::Sm)),
            xs_icon,
            spacing.step(SpaceStep::Xs),
        ),
        sm: metrics(
            sm_height,
            shapes.get(ShapeSize::Md),
            typography.get(TypographyRole::Body),
            Padding::ZERO
                .vertical(spacing.step(SpaceStep::Xs))
                .horizontal(spacing.step(SpaceStep::Md)),
            sm_icon,
            spacing.step(SpaceStep::Xs),
        ),
        md: metrics(
            md_height,
            shapes.get(ShapeSize::Lg),
            typography.get(TypographyRole::Body),
            Padding::ZERO
                .vertical(spacing.step(SpaceStep::Sm))
                .horizontal(spacing.step(SpaceStep::Lg)),
            md_icon,
            spacing.step(SpaceStep::Sm),
        ),
        lg: metrics(
            lg_height,
            shapes.get(ShapeSize::Lg),
            typography.get(TypographyRole::Heading),
            Padding::ZERO
                .vertical(spacing.step(SpaceStep::Md))
                .horizontal(spacing.step(SpaceStep::Xl)),
            lg_icon,
            spacing.step(SpaceStep::Sm),
        ),
    }
}

impl ControlMetricsScale {
    pub fn get(self, size: ControlSize) -> ControlMetrics {
        match size {
            ControlSize::Xs => self.xs,
            ControlSize::Sm => self.sm,
            ControlSize::Md => self.md,
            ControlSize::Lg => self.lg,
        }
    }
}

fn metrics(
    height: f32,
    shape: ShapeSpec,
    text_style: TextStyle,
    padding: Padding,
    icon_size: f32,
    gap: f32,
) -> ControlMetrics {
    ControlMetrics {
        height,
        shape,
        text_style,
        padding,
        radius: shape.radius_value(),
        font_size: text_style.size,
        icon_size,
        gap,
    }
}

#[cfg(test)]
mod component_tests {
    use super::*;

    #[test]
    fn control_height_grows_with_size() {
        assert!(control_metrics(ControlSize::Xs).height < control_metrics(ControlSize::Sm).height);
        assert!(control_metrics(ControlSize::Sm).height < control_metrics(ControlSize::Md).height);
        assert!(control_metrics(ControlSize::Md).height < control_metrics(ControlSize::Lg).height);
    }

    #[test]
    fn control_metrics_derive_legacy_radius_and_font_size_from_specs() {
        for size in [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ] {
            let metrics = control_metrics(size);

            assert_eq!(metrics.radius, metrics.shape.radius_value());
            assert_eq!(metrics.font_size, metrics.text_style.size);
        }
    }

    #[test]
    fn control_padding_is_symmetric_and_horizontal_first() {
        for size in [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ] {
            let padding = control_metrics(size).padding;

            assert_eq!(padding.left, padding.right);
            assert_eq!(padding.top, padding.bottom);
            assert!(padding.left > padding.top);
        }
    }

    #[test]
    fn control_size_sm_remains_small_across_densities() {
        let shapes = super::super::shape::scale();
        let typography = super::super::typography::scale();

        for density in ThemeDensity::ALL {
            let spacing = super::super::spacing::scale_for_density(density);
            let controls = scale_for_density(density, shapes, typography, spacing);
            let sm_height = controls.get(ControlSize::Sm).height;

            assert!(
                sm_height < controls.get(ControlSize::Md).height,
                "Sm height {} not less than Md height for density {:?}",
                sm_height,
                density
            );
            assert!(
                sm_height > controls.get(ControlSize::Xs).height,
                "Sm height {} not greater than Xs height for density {:?}",
                sm_height,
                density
            );
        }
    }

    #[test]
    fn density_control_heights_are_ordered_compact_lt_standard_lt_comfortable() {
        let shapes = super::super::shape::scale();
        let typography = super::super::typography::scale();

        let compact_spacing = super::super::spacing::scale_for_density(ThemeDensity::Compact);
        let standard_spacing = super::super::spacing::scale_for_density(ThemeDensity::Standard);
        let comfortable_spacing =
            super::super::spacing::scale_for_density(ThemeDensity::Comfortable);

        let compact = scale_for_density(ThemeDensity::Compact, shapes, typography, compact_spacing);
        let standard =
            scale_for_density(ThemeDensity::Standard, shapes, typography, standard_spacing);
        let comfortable = scale_for_density(
            ThemeDensity::Comfortable,
            shapes,
            typography,
            comfortable_spacing,
        );

        for size in [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ] {
            assert!(
                compact.get(size).height <= standard.get(size).height,
                "compact {:?} height {} > standard {}",
                size,
                compact.get(size).height,
                standard.get(size).height
            );
            assert!(
                standard.get(size).height <= comfortable.get(size).height,
                "standard {} height > comfortable {:?} {}",
                standard.get(size).height,
                size,
                comfortable.get(size).height
            );
        }
    }
}

use iced::Padding;

use super::shape::{ShapeRole, ShapeScale, ShapeSpec};
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
    ControlMetricsScale {
        xs: metrics(
            24.0,
            shapes.get(ShapeRole::Small),
            typography.get(TypographyRole::BodySmall),
            Padding::ZERO
                .vertical(spacing.step(SpaceStep::Xxs))
                .horizontal(spacing.step(SpaceStep::Sm)),
            12.0,
            spacing.step(SpaceStep::Xs),
        ),
        sm: metrics(
            28.0,
            shapes.get(ShapeRole::Medium),
            typography.get(TypographyRole::Body),
            Padding::ZERO
                .vertical(spacing.step(SpaceStep::Xs))
                .horizontal(spacing.step(SpaceStep::Md)),
            14.0,
            spacing.step(SpaceStep::Xs),
        ),
        md: metrics(
            32.0,
            shapes.get(ShapeRole::Large),
            typography.get(TypographyRole::Body),
            Padding::ZERO
                .vertical(spacing.step(SpaceStep::Sm))
                .horizontal(spacing.step(SpaceStep::Lg)),
            14.0,
            spacing.step(SpaceStep::Sm),
        ),
        lg: metrics(
            36.0,
            shapes.get(ShapeRole::Large),
            typography.get(TypographyRole::Heading),
            Padding::ZERO
                .vertical(spacing.step(SpaceStep::Md))
                .horizontal(spacing.step(SpaceStep::Xl)),
            16.0,
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
}

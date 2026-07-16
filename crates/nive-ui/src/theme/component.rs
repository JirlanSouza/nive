use iced::{Padding, Rectangle};

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

/// Form-specific projection of concrete theme control metrics.
///
/// The projection keeps the theme-owned outer geometry while replacing the
/// generic control text scale with fixed single-line form typography.
/// Built-in form controls never use an outer height below 20 px. Custom themes
/// should observe the same minimum for conforming form controls; finite custom
/// heights below it are still preserved so theme resolution is never silently
/// rewritten.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormControlMetrics {
    pub height: f32,
    pub shape: ShapeSpec,
    pub padding: Padding,
    pub radius: f32,
    pub icon_size: f32,
    pub gap: f32,
    pub text_style: TextStyle,
    pub strong_text_style: TextStyle,
    pub field_border_width: f32,
    pub focus_stroke_width: f32,
    pub focus_inset: f32,
    pub focus_radius: f32,
    pub content_inset: f32,
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

pub fn form_control_metrics(size: ControlSize) -> FormControlMetrics {
    form_metrics(control_metrics(size), super::typography::scale())
}

pub(crate) fn form_metrics(
    control: ControlMetrics,
    typography: TypographyScale,
) -> FormControlMetrics {
    const FIELD_BORDER_WIDTH: f32 = 1.0;
    const FOCUS_STROKE_WIDTH: f32 = 2.0;
    const FOCUS_INSET: f32 = 1.0;

    let text_style = typography.get(TypographyRole::Control);
    let strong_text_style = typography.get(TypographyRole::ControlStrong);
    let line_box = text_style.size * text_style.line_height;
    let content_inset = finite_nonnegative((control.height - line_box) / 2.0);
    let focus_radius = finite_nonnegative(control.radius - FOCUS_INSET);

    FormControlMetrics {
        height: control.height,
        shape: control.shape,
        padding: Padding {
            top: content_inset,
            right: control.padding.right,
            bottom: content_inset,
            left: control.padding.left,
        },
        radius: control.radius,
        icon_size: control.icon_size,
        gap: control.gap,
        text_style,
        strong_text_style,
        field_border_width: FIELD_BORDER_WIDTH,
        focus_stroke_width: FOCUS_STROKE_WIDTH,
        focus_inset: FOCUS_INSET,
        focus_radius,
        content_inset,
    }
}

impl FormControlMetrics {
    pub fn focus_bounds(self, bounds: Rectangle) -> Rectangle {
        let inset = finite_nonnegative(self.focus_inset);
        Rectangle {
            x: bounds.x + inset,
            y: bounds.y + inset,
            width: (bounds.width - inset * 2.0).max(0.0),
            height: (bounds.height - inset * 2.0).max(0.0),
        }
    }
}

fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
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
    use crate::theme::typography;

    #[test]
    fn control_height_grows_with_size() {
        assert!(control_metrics(ControlSize::Xs).height < control_metrics(ControlSize::Sm).height);
        assert!(control_metrics(ControlSize::Sm).height < control_metrics(ControlSize::Md).height);
        assert!(control_metrics(ControlSize::Md).height < control_metrics(ControlSize::Lg).height);
    }

    #[test]
    fn form_control_metrics_preserve_geometry_and_supply_form_typography() {
        for size in [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ] {
            let control = control_metrics(size);
            let form = form_control_metrics(size);

            assert_eq!(form.height, control.height);
            assert_eq!(form.shape, control.shape);
            assert_eq!(form.padding.left, control.padding.left);
            assert_eq!(form.padding.right, control.padding.right);
            assert_eq!(form.icon_size, control.icon_size);
            assert_eq!(form.gap, control.gap);
            assert_eq!(form.text_style, typography(TypographyRole::Control));
            assert_eq!(
                form.strong_text_style,
                typography(TypographyRole::ControlStrong)
            );
            assert_eq!(form.field_border_width, 1.0);
            assert_eq!(form.focus_stroke_width, 2.0);
            assert_eq!(form.focus_inset, 1.0);
            assert_eq!(form.focus_radius, (form.radius - 1.0).max(0.0));
        }
    }

    #[test]
    fn focus_bounds_are_inset_without_reserving_layout_space() {
        let form = form_control_metrics(ControlSize::Sm);
        let outer = Rectangle::new(
            iced::Point::new(4.0, 6.0),
            iced::Size::new(100.0, form.height),
        );

        assert_eq!(
            form.focus_bounds(outer),
            Rectangle::new(
                iced::Point::new(5.0, 7.0),
                iced::Size::new(98.0, form.height - 2.0),
            )
        );
    }

    #[test]
    fn form_control_metric_computed_insets_are_finite_and_radius_is_clamped() {
        let mut control = control_metrics(ControlSize::Xs);
        control.height = f32::NAN;
        control.radius = -4.0;
        let form = form_metrics(control, super::super::typography::scale());

        assert!(form.height.is_nan());
        assert_eq!(form.content_inset, 0.0);
        assert_eq!(form.focus_radius, 0.0);

        control.height = 18.0;
        control.radius = f32::INFINITY;
        let form = form_metrics(control, super::super::typography::scale());

        assert_eq!(form.height, 18.0);
        assert!(form.content_inset.is_finite());
        assert!(form.content_inset >= 0.0);
        assert_eq!(form.focus_radius, 0.0);
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

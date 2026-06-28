use std::time::Duration;

use nive::prelude::*;
use nive::ui::theme::{SurfaceRole, ToneRole};
use nive::ui::widgets::text as ntext;
use nive::widget::column;

use crate::app::{Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid, variant_row};

pub fn view(_app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Motion,
        column![
            section("AnimatedVisual", animated_visual()),
            section("AnimatedLayout", animated_layout()),
            section("Animation data", timeline()),
        ]
        .spacing(18),
    )
}

fn animated_visual() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Rotation",
            AnimatedVisual::new(|frame| {
                Icon::new(IconName::RefreshCw)
                    .size(28.0)
                    .rotation(Radians(frame.turns() * std::f32::consts::TAU))
                    .into()
            })
            .animation(Animation::linear(Duration::from_millis(1500)).repeat()),
        ),
        example_cell(
            "Pulse",
            AnimatedVisual::new(|frame| {
                let alpha = 0.35 + frame.progress() * 0.65;
                Icon::new(IconName::Info)
                    .size(28.0)
                    .color(
                        theme::active()
                            .tone(ToneRole::Info)
                            .color
                            .scale_alpha(alpha),
                    )
                    .into()
            })
            .animation(
                Animation::linear(Duration::from_millis(900))
                    .easing(Easing::EaseInOut)
                    .auto_reverse()
                    .repeat(),
            ),
        ),
    ])
}

fn animated_layout() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Reveal",
            AnimatedLayout::height(
                Panel::new(ntext::body("Animated layout content"))
                    .role(SurfaceRole::Elevated)
                    .padding(14),
                true,
                20.0,
                72.0,
            )
            .duration(Duration::from_millis(1200))
            .easing(Easing::EaseInOut),
        ),
        example_cell(
            "StaggeredPulse",
            variant_row((0..5).map(|index| {
                AnimatedVisual::new(move |frame| {
                    let pulse = StaggeredPulse::new(5)
                        .overlap(0.65)
                        .rest(0.2)
                        .activity(frame, index);
                    Icon::new(IconName::CheckCircle)
                        .size(18.0)
                        .color(
                            theme::active()
                                .tone(ToneRole::Primary)
                                .color
                                .scale_alpha(pulse),
                        )
                        .into()
                })
                .animation(Animation::linear(Duration::from_millis(1200)).repeat())
                .into()
            })),
        ),
    ])
}

fn timeline() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Animation",
            column![
                ntext::body("linear, ease-in, ease-out, and ease-in-out constructors"),
                ntext::caption("repeat() is used by visual examples above"),
            ]
            .spacing(6),
        ),
        example_cell(
            "AnimationFrame",
            column![
                ntext::body_small(
                    "AnimationFrame is produced by AnimatedVisual and AnimatedLayout."
                ),
                ntext::body_small("Views use progress(), turns(), cycle(), elapsed(), and lerp()."),
            ]
            .spacing(6),
        ),
        example_cell(
            "Easing",
            variant_row([
                Badge::neutral("Linear").into(),
                Badge::info("EaseIn").into(),
                Badge::success("EaseOut").into(),
                Badge::warning("EaseInOut").into(),
            ]),
        ),
    ])
}

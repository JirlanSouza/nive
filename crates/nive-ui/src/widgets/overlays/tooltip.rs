mod scope;
mod widget;

use std::{borrow::Cow, time::Duration};

use iced::{
    border::Radius,
    widget::{container, text},
    Background, Border, Length, Padding,
};

use crate::{
    theme::{self, BorderRole, SurfaceRole, TypographyRole},
    Element,
};

use self::widget::TooltipWidget;
pub use scope::TooltipScope;

const COLD_DELAY: Duration = Duration::from_millis(500);
const TOOLTIP_MAX_WIDTH: f32 = 280.0;

/// Preferred physical side for passive Tooltip disclosure.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipPlacement {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

/// Passive text disclosure anchored to one widget.
///
/// Tooltip text supplements rather than replaces the anchor's semantic name.
/// It reveals after 500ms when isolated, uses scoped neighboring timing inside
/// [`TooltipScope`], and emits no native accessibility node.
pub struct Tooltip<'a, Message> {
    anchor: Element<'a, Message>,
    label: Cow<'a, str>,
    placement: TooltipPlacement,
    delay: Duration,
    now_override: Option<iced::time::Instant>,
    intent_override: Option<(bool, bool)>,
}

impl<'a, Message> Tooltip<'a, Message>
where
    Message: 'a,
{
    pub fn new(anchor: impl Into<Element<'a, Message>>, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            anchor: anchor.into(),
            label: label.into(),
            placement: TooltipPlacement::default(),
            delay: COLD_DELAY,
            now_override: None,
            intent_override: None,
        }
    }

    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    #[cfg(test)]
    fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    #[cfg(test)]
    fn at(mut self, now: iced::time::Instant) -> Self {
        self.now_override = Some(now);
        self
    }

    #[cfg(test)]
    fn intent(mut self, hovered: bool, focused: bool) -> Self {
        self.intent_override = Some((hovered, focused));
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let label = surface(self.label);
        Element::new(TooltipWidget::new(
            self.anchor,
            label,
            self.placement,
            self.delay,
            self.now_override,
            self.intent_override,
        ))
    }
}

impl<'a, Message> From<Tooltip<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(tooltip: Tooltip<'a, Message>) -> Self {
        tooltip.into_element()
    }
}

/// Compatibility forwarder for the first release containing [`Tooltip`].
#[deprecated(
    since = "0.1.0",
    note = "use Tooltip::new(anchor, label); this forwarder is removed in the next published release"
)]
pub fn bottom<'a, Message>(
    anchor: impl Into<Element<'a, Message>>,
    label: impl Into<Cow<'a, str>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    Tooltip::new(anchor, label)
        .placement(TooltipPlacement::Bottom)
        .into()
}

#[cfg(test)]
pub(crate) fn bottom_without_delay<'a, Message>(
    anchor: impl Into<Element<'a, Message>>,
    label: impl Into<Cow<'a, str>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    Tooltip::new(anchor, label)
        .placement(TooltipPlacement::Bottom)
        .delay(Duration::ZERO)
        .into()
}

fn surface<'a, Message>(label: Cow<'a, str>) -> Element<'a, Message>
where
    Message: 'a,
{
    let typography = theme::typography(TypographyRole::BodySmall);
    let label = text(label)
        .size(typography.size)
        .line_height(typography.line_height)
        .shaping(text::Shaping::Auto)
        .wrapping(text::Wrapping::WordOrGlyph)
        .width(Length::Shrink);

    container(label)
        .padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 4.0,
            left: 8.0,
        })
        .max_width(TOOLTIP_MAX_WIDTH)
        .style(surface_style)
        .into()
}

fn surface_style(theme: &crate::theme::Theme) -> container::Style {
    let theme = *theme;
    let surface = theme.surface(SurfaceRole::Popover);
    let perimeter = theme.border(BorderRole::Subtle);

    container::Style {
        text_color: Some(surface.foreground),
        background: Some(Background::Color(surface.background)),
        border: Border {
            color: perimeter.color,
            width: 1.0,
            radius: Radius::new(4.0),
        },
        shadow: surface.shadow,
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tooltip_tests {
    use std::time::Duration;

    use iced::{
        advanced::widget::operation,
        keyboard::{self, key},
        widget::{column, container, row, Id},
        Event, Point, Size,
    };

    use super::*;
    use crate::{
        test_support::WidgetHarness,
        theme::Theme,
        widgets::{button, Input},
    };

    #[test]
    fn defaults_and_surface_metrics_are_exact() {
        let tooltip = Tooltip::<()>::new(iced::widget::Space::new(), "Help");
        let style = surface_style(&Theme::Dark);

        assert_eq!(tooltip.placement, TooltipPlacement::Bottom);
        assert_eq!(tooltip.delay, Duration::from_millis(500));
        assert_eq!(style.border.width, 1.0);
        assert_eq!(style.border.radius, Radius::new(4.0));
    }

    #[test]
    fn isolated_tooltip_waits_five_hundred_milliseconds() {
        let start = iced::time::Instant::now();
        let mut harness =
            WidgetHarness::new(tooltip_at(start, 1, true, false), Size::new(320.0, 120.0));
        harness.update(redraw(start));
        assert_eq!(visible_keys(&mut harness), Vec::<u64>::new());

        harness.replace(tooltip_at(
            start + Duration::from_millis(499),
            1,
            true,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(499)));
        assert!(visible_keys(&mut harness).is_empty());

        harness.replace(tooltip_at(
            start + Duration::from_millis(500),
            1,
            true,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(500)));
        assert_eq!(visible_keys(&mut harness).len(), 1);
    }

    #[test]
    fn scoped_neighbor_uses_warm_delay_and_pointer_wins() {
        let start = iced::time::Instant::now();
        let mut harness = WidgetHarness::new(
            scope_at(start, true, false, false, true),
            Size::new(320.0, 120.0),
        );
        harness.update(redraw(start));
        harness.replace(scope_at(
            start + Duration::from_millis(500),
            true,
            false,
            false,
            true,
        ));
        harness.update(redraw(start + Duration::from_millis(500)));
        assert_eq!(visible_keys(&mut harness), vec![1]);

        harness.replace(scope_at(
            start + Duration::from_millis(550),
            false,
            true,
            true,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(550)));
        assert!(visible_keys(&mut harness).is_empty());

        harness.replace(scope_at(
            start + Duration::from_millis(650),
            false,
            true,
            true,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(650)));
        assert_eq!(visible_keys(&mut harness), vec![2]);
    }

    #[test]
    fn same_key_reentry_uses_cold_delay() {
        let start = iced::time::Instant::now();
        let mut harness = WidgetHarness::new(
            scope_at(start, true, false, false, false),
            Size::new(320.0, 120.0),
        );
        harness.update(redraw(start));
        harness.replace(scope_at(
            start + Duration::from_millis(500),
            true,
            false,
            false,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(500)));
        assert_eq!(visible_keys(&mut harness), vec![1]);

        harness.replace(scope_at(
            start + Duration::from_millis(550),
            false,
            false,
            false,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(550)));
        harness.replace(scope_at(
            start + Duration::from_millis(600),
            true,
            false,
            false,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(600)));
        harness.replace(scope_at(
            start + Duration::from_millis(1_099),
            true,
            false,
            false,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(1_099)));
        assert!(visible_keys(&mut harness).is_empty());

        harness.replace(scope_at(
            start + Duration::from_millis(1_100),
            true,
            false,
            false,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(1_100)));
        assert_eq!(visible_keys(&mut harness), vec![1]);
    }

    #[test]
    fn an_unshown_candidate_does_not_warm_its_neighbor() {
        let start = iced::time::Instant::now();
        let mut harness = WidgetHarness::new(
            scope_at(start, true, false, false, false),
            Size::new(320.0, 120.0),
        );
        harness.update(redraw(start));
        harness.replace(scope_at(
            start + Duration::from_millis(200),
            false,
            true,
            false,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(200)));
        harness.replace(scope_at(
            start + Duration::from_millis(699),
            false,
            true,
            false,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(699)));
        assert!(visible_keys(&mut harness).is_empty());

        harness.replace(scope_at(
            start + Duration::from_millis(700),
            false,
            true,
            false,
            false,
        ));
        harness.update(redraw(start + Duration::from_millis(700)));
        assert_eq!(visible_keys(&mut harness), vec![2]);
    }

    #[test]
    fn nested_scopes_and_independent_trees_keep_separate_sessions() {
        let start = iced::time::Instant::now();
        let mut nested = WidgetHarness::new(nested_at(start), Size::new(320.0, 120.0));
        nested.update(redraw(start));
        nested.replace(nested_at(start + Duration::from_millis(500)));
        nested.update(redraw(start + Duration::from_millis(500)));
        assert_eq!(visible_keys(&mut nested), vec![1, 2]);

        let mut independent = WidgetHarness::new(
            scope_at(
                start + Duration::from_millis(500),
                true,
                false,
                false,
                false,
            ),
            Size::new(320.0, 120.0),
        );
        independent.update(redraw(start + Duration::from_millis(500)));
        independent.replace(scope_at(
            start + Duration::from_millis(600),
            true,
            false,
            false,
            false,
        ));
        independent.update(redraw(start + Duration::from_millis(600)));
        assert!(visible_keys(&mut independent).is_empty());
    }

    #[test]
    fn every_preferred_side_keeps_the_four_pixel_gap() {
        let start = iced::time::Instant::now();
        for placement in [
            TooltipPlacement::Top,
            TooltipPlacement::Right,
            TooltipPlacement::Bottom,
            TooltipPlacement::Left,
        ] {
            let tooltip: Element<'_, ()> = container(
                Tooltip::new(iced::widget::Space::new().width(20).height(20), "Help")
                    .placement(placement)
                    .delay(Duration::ZERO)
                    .at(start)
                    .intent(true, false),
            )
            .padding(100)
            .into();
            let mut harness = WidgetHarness::new(tooltip, Size::new(400.0, 300.0));
            harness.update(redraw(start));
            let bounds = harness.overlay_bounds().expect("visible Tooltip");

            match placement {
                TooltipPlacement::Top => assert_eq!(bounds.y + bounds.height, 96.0),
                TooltipPlacement::Right => assert_eq!(bounds.x, 124.0),
                TooltipPlacement::Bottom => assert_eq!(bounds.y, 124.0),
                TooltipPlacement::Left => assert_eq!(bounds.x + bounds.width, 96.0),
            }
        }
    }

    #[test]
    fn long_text_wraps_within_the_tooltip_and_safe_viewport_caps() {
        let start = iced::time::Instant::now();
        let tooltip: Element<'_, ()> = Tooltip::new(
            iced::widget::Space::new().width(20).height(20),
            "A deliberately long tooltip label that must wrap instead of widening beyond its compact desktop disclosure limit.",
        )
        .delay(Duration::ZERO)
        .at(start)
        .intent(true, false)
        .into();
        let mut harness = WidgetHarness::new(tooltip, Size::new(240.0, 180.0));
        harness.update(redraw(start));
        let bounds = harness.overlay_bounds().expect("visible Tooltip");

        assert!(bounds.width <= 224.0);
        assert!(bounds.height > 24.0);
    }

    #[test]
    fn focus_reveals_and_escape_suppresses_until_intent_leaves() {
        let start = iced::time::Instant::now();
        let id = Id::unique();
        let tooltip: Element<'_, ()> =
            Tooltip::new(Input::new("Anchor", "").id(id.clone()), "Help")
                .delay(Duration::ZERO)
                .at(start)
                .into();
        let mut harness = WidgetHarness::new(tooltip, Size::new(320.0, 120.0));
        harness.focus(id);
        harness.update(redraw(start));
        assert_eq!(visible_keys(&mut harness).len(), 1);

        harness.update(key_pressed(key::Named::Escape, key::Code::Escape));
        assert!(visible_keys(&mut harness).is_empty());
    }

    #[test]
    fn disabled_anchor_remains_pointer_explainable_without_focusability() {
        let start = iced::time::Instant::now();
        let anchor = button::secondary("Unavailable").on_press(()).disabled(true);
        let tooltip: Element<'_, ()> = Tooltip::new(anchor, "Not available in this state")
            .delay(Duration::ZERO)
            .at(start)
            .into();
        let mut harness = WidgetHarness::new(tooltip, Size::new(320.0, 120.0));
        harness.set_cursor(Point::new(10.0, 10.0));
        harness.update(redraw(start));

        assert_eq!(harness.focused_widgets(), 0);
        assert_eq!(visible_keys(&mut harness).len(), 1);
    }

    #[test]
    fn collision_flips_and_shifts_inside_the_safe_viewport() {
        let start = iced::time::Instant::now();
        let tooltip: Element<'_, ()> = column![
            iced::widget::Space::new().height(160),
            Tooltip::new(
                iced::widget::Space::new().width(20).height(20),
                "A wide tooltip that must remain inside the safe viewport",
            )
            .delay(Duration::ZERO)
            .at(start)
            .intent(true, false),
        ]
        .into();
        let mut harness = WidgetHarness::new(tooltip, Size::new(220.0, 200.0));
        harness.update(redraw(start));
        let bounds = harness.overlay_bounds().expect("visible Tooltip");

        assert!(bounds.y + bounds.height <= 156.0);
        assert!(bounds.x >= 8.0);
        assert!(bounds.x + bounds.width <= 212.0);
    }

    #[test]
    fn pointer_leave_closes_without_intercepting_anchor_events() {
        let start = iced::time::Instant::now();
        let tooltip: Element<'_, ()> = Tooltip::new(
            iced::widget::Space::new().width(20).height(20),
            "Pointer help",
        )
        .delay(Duration::ZERO)
        .at(start)
        .into();
        let mut harness = WidgetHarness::new(tooltip, Size::new(320.0, 120.0));
        harness.set_cursor(Point::new(10.0, 10.0));
        let entered = harness.update(redraw(start));
        assert!(!entered.captured);
        assert_eq!(visible_keys(&mut harness).len(), 1);

        harness.set_cursor(Point::new(100.0, 100.0));
        let left = harness.update(redraw(start + Duration::from_millis(1)));
        assert!(!left.captured);
        assert!(visible_keys(&mut harness).is_empty());
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_bottom_forwarder_remains_source_compatible() {
        let element: Element<'_, ()> = bottom(iced::widget::Space::new(), "Legacy help");
        let harness = WidgetHarness::new(element, Size::new(200.0, 80.0));

        assert_eq!(harness.bounds().position(), Point::ORIGIN);
    }

    fn tooltip_at(
        now: iced::time::Instant,
        key: u64,
        hovered: bool,
        focused: bool,
    ) -> Element<'static, ()> {
        Tooltip::new(
            iced::widget::Space::new().width(20).height(20),
            key.to_string(),
        )
        .at(now)
        .intent(hovered, focused)
        .into()
    }

    fn scope_at(
        now: iced::time::Instant,
        first_hovered: bool,
        second_hovered: bool,
        first_focused: bool,
        second_focused: bool,
    ) -> Element<'static, ()> {
        TooltipScope::new(row![
            Tooltip::new(iced::widget::Space::new().width(20).height(20), "First")
                .at(now)
                .intent(first_hovered, first_focused),
            Tooltip::new(iced::widget::Space::new().width(20).height(20), "Second")
                .at(now)
                .intent(second_hovered, second_focused),
        ])
        .at(now)
        .into()
    }

    fn nested_at(now: iced::time::Instant) -> Element<'static, ()> {
        TooltipScope::new(row![
            Tooltip::new(iced::widget::Space::new().width(20).height(20), "Outer")
                .at(now)
                .intent(true, false),
            TooltipScope::new(
                Tooltip::new(iced::widget::Space::new().width(20).height(20), "Inner")
                    .at(now)
                    .intent(true, false),
            )
            .at(now),
        ])
        .at(now)
        .into()
    }

    fn redraw(now: iced::time::Instant) -> Event {
        Event::Window(iced::window::Event::RedrawRequested(now))
    }

    fn key_pressed(named: key::Named, code: key::Code) -> Event {
        let key = keyboard::Key::Named(named);
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: key::Physical::Code(code),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: None,
            repeat: false,
        })
    }

    fn visible_keys(harness: &mut WidgetHarness<'_, ()>) -> Vec<u64> {
        struct VisibleKeys {
            index: u64,
            visible: Vec<u64>,
        }

        impl operation::Operation for VisibleKeys {
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
                operate(self);
            }

            fn custom(
                &mut self,
                _id: Option<&iced::advanced::widget::Id>,
                _bounds: iced::Rectangle,
                state: &mut dyn std::any::Any,
            ) {
                if let Some(state) = state.downcast_ref::<widget::TooltipState>() {
                    self.index += 1;
                    if state.visible {
                        self.visible.push(self.index);
                    }
                }
            }
        }

        let mut keys = VisibleKeys {
            index: 0,
            visible: Vec::new(),
        };
        harness.operate(&mut keys);
        keys.visible
    }

    #[test]
    fn real_pointer_intent_is_observed_without_capturing_activation() {
        let start = iced::time::Instant::now();
        let tooltip: Element<'_, ()> = Tooltip::new(
            iced::widget::Space::new().width(20).height(20),
            "Pointer help",
        )
        .delay(Duration::ZERO)
        .at(start)
        .into();
        let mut harness = WidgetHarness::new(tooltip, Size::new(320.0, 120.0));
        harness.set_cursor(Point::new(10.0, 10.0));
        let result = harness.update(redraw(start));

        assert!(!result.captured);
        assert_eq!(visible_keys(&mut harness).len(), 1);
    }
}

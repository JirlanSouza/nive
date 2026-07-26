mod chrome;
mod content;
mod style;

use std::borrow::Cow;

use iced::{border::Radius, widget::Id, Length, Padding};

use crate::theme::ControlSize;
use crate::Element;

use self::{
    chrome::ButtonChrome,
    content::{Content, TextAlign},
};
use crate::advanced::pressable::{FocusRingPlacement, Pressable};
use crate::widgets::overlays::tooltip as tooltip_widget;
use crate::IconRef;

pub use style::{button_control_state, focus_ring, ButtonFocusRing};
pub use style::{ButtonIntent, ButtonVariant};

/// Action button with separate intent and visual variant axes.
pub struct Button<'a, Message> {
    content: Content<'a, Message>,
    leading_icon: Option<IconRef>,
    trailing_icon: Option<IconRef>,
    intent: ButtonIntent,
    variant: ButtonVariant,
    size: ControlSize,
    text_align: TextAlign,
    width: Option<Length>,
    height: Option<Length>,
    padding: Option<Padding>,
    loading: bool,
    reserve_loading_indicator: bool,
    disabled: bool,
    focusable: bool,
    id: Option<Id>,
    on_press: Option<Message>,
    semantic_name: Option<Cow<'a, str>>,
    tooltip: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GroupedItemKind {
    Embedded,
    Toolbar,
}

/// How a selected grouped item announces itself.
///
/// `Outlined` is the shared treatment: selected fill plus the accent border the
/// theme resolves for `ControlRole::Selectable`. `Flat` drops the border for
/// chrome that already carries a directional selection marker of its own — a
/// navigation rail anchors selection to the panel-facing edge, so a closed
/// outline would be a second accent for the same state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectionChrome {
    Outlined,
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GroupedItemSpec {
    pub(crate) size: ControlSize,
    pub(crate) radius: Radius,
    pub(crate) height: f32,
    pub(crate) padding_h: f32,
    pub(crate) selected: bool,
    pub(crate) selection: SelectionChrome,
    pub(crate) destructive: bool,
    pub(crate) kind: GroupedItemKind,
}

/// Creates a suggested solid button.
pub fn primary<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Label(label.into()),
        ButtonIntent::Suggested,
        ButtonVariant::Solid,
    )
}

/// Creates a neutral outline button.
pub fn outline<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Label(label.into()),
        ButtonIntent::Neutral,
        ButtonVariant::Outline,
    )
}

/// Creates a neutral outline secondary button.
///
/// Prefer at most one [`primary`] action in a local action group. Secondary
/// actions use Outline; contextual low-emphasis actions use [`tertiary`] or an
/// explicit advanced variant.
pub fn secondary<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Label(label.into()),
        ButtonIntent::Neutral,
        ButtonVariant::Outline,
    )
}

/// Creates a neutral ghost button.
pub fn ghost<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Label(label.into()),
        ButtonIntent::Neutral,
        ButtonVariant::Ghost,
    )
}

/// Creates a neutral ghost tertiary button.
pub fn tertiary<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    ghost(label)
}

/// Creates a destructive solid confirmation button.
///
/// Preparatory destructive actions should use the public intent/variant axes
/// with Outline or Ghost instead of competing with final confirmation.
pub fn destructive<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Label(label.into()),
        ButtonIntent::Destructive,
        ButtonVariant::Solid,
    )
}

/// Creates a neutral ghost icon button.
///
/// The semantic name is retained independently from optional tooltip content.
/// Omitting it is not a supported public construction:
///
/// ```compile_fail
/// use nive_ui::widgets::button;
/// use nive_ui::IconRole;
///
/// let _ = button::icon::<()>(IconRole::WindowClose);
/// ```
pub fn icon<'a, Message>(
    app_icon: impl Into<IconRef>,
    semantic_name: impl Into<Cow<'a, str>>,
) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    let mut button = Button::new(
        Content::Icon(app_icon.into()),
        ButtonIntent::Neutral,
        ButtonVariant::Ghost,
    );
    button.semantic_name = Some(semantic_name.into());
    button
}

impl<'a, Message> Button<'a, Message>
where
    Message: Clone + 'a,
{
    fn new(content: Content<'a, Message>, intent: ButtonIntent, variant: ButtonVariant) -> Self {
        Self {
            content,
            leading_icon: None,
            trailing_icon: None,
            intent,
            variant,
            size: ControlSize::Sm,
            text_align: TextAlign::Center,
            width: None,
            height: None,
            padding: None,
            loading: false,
            reserve_loading_indicator: false,
            disabled: false,
            focusable: true,
            id: None,
            on_press: None,
            semantic_name: None,
            tooltip: None,
        }
    }

    /// Sets the visual variant axis.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the semantic intent axis.
    pub fn intent(mut self, intent: ButtonIntent) -> Self {
        self.intent = intent;
        self
    }

    pub(crate) fn custom(content: Element<'a, Message>) -> Self {
        Self::new(
            Content::Custom(content),
            ButtonIntent::Neutral,
            ButtonVariant::Ghost,
        )
    }

    /// Sets this button to `Suggested + Solid`.
    pub fn primary(self) -> Self {
        self.intent(ButtonIntent::Suggested)
            .variant(ButtonVariant::Solid)
    }

    /// Sets this button to `Neutral + Outline`.
    pub fn secondary(self) -> Self {
        self.intent(ButtonIntent::Neutral)
            .variant(ButtonVariant::Outline)
    }

    /// Sets this button to `Neutral + Ghost`.
    pub fn tertiary(self) -> Self {
        self.intent(ButtonIntent::Neutral)
            .variant(ButtonVariant::Ghost)
    }

    /// Sets this button to `Neutral + Outline`.
    pub fn outline(self) -> Self {
        self.intent(ButtonIntent::Neutral)
            .variant(ButtonVariant::Outline)
    }

    /// Sets this button to `Neutral + Ghost`.
    pub fn ghost(self) -> Self {
        self.intent(ButtonIntent::Neutral)
            .variant(ButtonVariant::Ghost)
    }

    /// Sets this button to `Destructive + Solid`.
    ///
    /// Use `destructive` for actions. Status widgets use `danger` for the
    /// corresponding tone language.
    pub fn destructive(self) -> Self {
        self.intent(ButtonIntent::Destructive)
            .variant(ButtonVariant::Solid)
    }

    /// Sets the intent axis to neutral.
    pub fn neutral(self) -> Self {
        self.intent(ButtonIntent::Neutral)
    }

    /// Sets the intent axis to suggested/default action.
    pub fn suggested(self) -> Self {
        self.intent(ButtonIntent::Suggested)
    }

    /// Sets the variant axis to solid.
    pub fn solid(self) -> Self {
        self.variant(ButtonVariant::Solid)
    }

    /// Sets the variant axis to subtle.
    pub fn subtle(self) -> Self {
        self.variant(ButtonVariant::Subtle)
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(mut self) -> Self {
        self.size = ControlSize::Xs;
        self
    }

    pub fn sm(mut self) -> Self {
        self.size = ControlSize::Sm;
        self
    }

    pub fn md(mut self) -> Self {
        self.size = ControlSize::Md;
        self
    }

    pub fn lg(mut self) -> Self {
        self.size = ControlSize::Lg;
        self
    }

    pub fn align_start(mut self) -> Self {
        self.text_align = TextAlign::Start;
        self
    }

    pub fn align_center(mut self) -> Self {
        self.text_align = TextAlign::Center;
        self
    }

    crate::impl_layout_builders!(width_opt, fill_width_opt, shrink_width_opt);

    pub(crate) fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn leading_icon(mut self, icon: impl Into<IconRef>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    pub fn trailing_icon(mut self, icon: impl Into<IconRef>) -> Self {
        self.trailing_icon = Some(icon.into());
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self.reserve_loading_indicator = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn semantic_name_value(&self) -> Option<&str> {
        self.semantic_name.as_deref()
    }

    pub fn tooltip_maybe<T>(mut self, tooltip: Option<T>) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.tooltip = tooltip.map(Into::into);
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    pub(crate) fn into_grouped_item(self, spec: GroupedItemSpec) -> Element<'a, Message> {
        self.into_grouped_item_with_placement(spec, FocusRingPlacement::Outset)
    }

    pub(crate) fn into_grouped_item_inset(self, spec: GroupedItemSpec) -> Element<'a, Message> {
        self.into_grouped_item_with_placement(spec, FocusRingPlacement::FormInset)
    }

    fn into_grouped_item_with_placement(
        mut self,
        spec: GroupedItemSpec,
        placement: FocusRingPlacement,
    ) -> Element<'a, Message> {
        self.size = spec.size;
        self.height = Some(Length::Fixed(spec.height));
        if self.width.is_none() {
            self.width = Some(match &self.content {
                Content::Icon(_) => Length::Fixed(spec.height),
                Content::Label(_) | Content::Custom(_) => Length::Shrink,
            });
        }
        if self.padding.is_none() {
            self.padding = Some(match &self.content {
                Content::Icon(_) => Padding::ZERO,
                Content::Label(_) | Content::Custom(_) => Padding::ZERO.horizontal(spec.padding_h),
            });
        }

        self.into_element_chrome_with_placement(ButtonChrome::Grouped(spec), placement)
    }

    fn into_element_chrome(self, chrome: ButtonChrome) -> Element<'a, Message> {
        self.into_element_chrome_with_placement(chrome, FocusRingPlacement::FormInset)
    }

    fn into_element_chrome_with_placement(
        mut self,
        chrome: ButtonChrome,
        placement: FocusRingPlacement,
    ) -> Element<'a, Message> {
        let radius = chrome.radius(self.size);
        let activation = self.keyboard_activation();
        let id = self.id.clone();
        let ring = self.focus_ring();
        let tooltip_label = self.tooltip.take();
        let button = chrome::into_app_button(self, chrome);
        let button: Element<'a, Message> = match activation {
            Some(message) => Pressable::new(button, message, id, radius, ring)
                .focus_placement(placement)
                .into(),
            None => button.into(),
        };

        match tooltip_label {
            Some(label) => tooltip_widget::Tooltip::new(button, label).into(),
            None => button,
        }
    }

    fn keyboard_activation(&self) -> Option<Message> {
        if self.focusable && !self.disabled && !self.loading {
            self.on_press.clone()
        } else {
            None
        }
    }

    fn focus_ring(&self) -> ButtonFocusRing {
        match self.variant {
            ButtonVariant::Solid if self.intent == ButtonIntent::Suggested => {
                ButtonFocusRing::OnPrimary
            }
            ButtonVariant::Solid if self.intent == ButtonIntent::Destructive => {
                ButtonFocusRing::OnDanger
            }
            _ => ButtonFocusRing::Default,
        }
    }
}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(button: Button<'a, Message>) -> Self {
        button.into_element_chrome(ButtonChrome::Standalone)
    }
}

#[cfg(test)]
mod button_tests {
    use super::*;
    use crate::test_support::{named_probe, WidgetHarness};
    use crate::theme::{ThemeBuilder, ThemeDensity, ThemeMode, TypographyRole};
    #[allow(unused_imports)]
    use crate::IconRole;
    use iced::{
        keyboard::{
            self,
            key::{Code, Named, Physical},
            Key, Location, Modifiers,
        },
        mouse, Event, Point, Size,
    };

    fn key_pressed(key: Key) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Code(Code::Enter),
            location: Location::Standard,
            modifiers: Modifiers::NONE,
            text: None,
            repeat: false,
        })
    }

    #[test]
    fn shrink_width_sets_button_width_to_shrink() {
        let button = secondary::<()>("Open").fill_width().shrink_width();

        assert_eq!(button.width, Some(Length::Shrink));
    }

    #[test]
    fn label_buttons_are_intrinsic_until_fill_width_is_requested() {
        let intrinsic = secondary::<()>("Open");
        assert_eq!(
            intrinsic.content.default_width(ControlSize::Sm),
            Length::Shrink
        );
        assert_eq!(intrinsic.width, None);

        let filled = secondary::<()>("Open").fill_width();
        assert_eq!(filled.width, Some(Length::Fill));
    }

    #[test]
    fn high_level_shortcuts_map_to_intent_and_variant_pairs() {
        assert_button(
            primary::<()>("Create"),
            ButtonIntent::Suggested,
            ButtonVariant::Solid,
        );
        assert_button(
            secondary::<()>("Copy"),
            ButtonIntent::Neutral,
            ButtonVariant::Outline,
        );
        assert_button(
            outline::<()>("Inspect"),
            ButtonIntent::Neutral,
            ButtonVariant::Outline,
        );
        assert_button(
            ghost::<()>("Preview"),
            ButtonIntent::Neutral,
            ButtonVariant::Ghost,
        );
        assert_button(
            tertiary::<()>("More"),
            ButtonIntent::Neutral,
            ButtonVariant::Ghost,
        );
        assert_button(
            destructive::<()>("Delete"),
            ButtonIntent::Destructive,
            ButtonVariant::Solid,
        );
        assert_button(
            icon::<()>(IconRole::WindowClose, "Close"),
            ButtonIntent::Neutral,
            ButtonVariant::Ghost,
        );
    }

    #[test]
    fn axis_shortcuts_change_their_button_axis() {
        let button = secondary::<()>("Save").suggested().solid();

        assert_button(button, ButtonIntent::Suggested, ButtonVariant::Solid);
        assert_button(
            ghost::<()>("Neutral").solid(),
            ButtonIntent::Neutral,
            ButtonVariant::Solid,
        );
        assert_button(
            primary::<()>("Subtle").neutral().subtle(),
            ButtonIntent::Neutral,
            ButtonVariant::Subtle,
        );
    }

    fn assert_button(button: Button<'_, ()>, intent: ButtonIntent, variant: ButtonVariant) {
        assert_eq!(button.intent, intent);
        assert_eq!(button.variant, variant);
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Message {
        Pressed,
    }

    #[test]
    fn disabled_button_ignores_present_callback() {
        let button = primary("Save").on_press(Message::Pressed).disabled(true);

        assert_eq!(button.keyboard_activation(), None);
    }

    #[test]
    fn tooltip_accepts_owned_label() {
        let button = icon::<()>(IconRole::WindowClose, "Close").tooltip(String::from("Dismiss"));

        assert_eq!(button.semantic_name_value(), Some("Close"));
        assert_eq!(button.tooltip.as_deref(), Some("Dismiss"));
    }

    #[test]
    fn labels_accept_owned_cow_and_use_control_strong_metrics() {
        let button = secondary::<()>(String::from("Owned label"));
        let Content::Label(label) = &button.content else {
            panic!("label content")
        };

        assert_eq!(label.as_ref(), "Owned label");
        assert_eq!(
            style::metrics(ControlSize::Xs).font_size,
            crate::theme::typography(TypographyRole::ControlStrong).size
        );
        assert_eq!(style::metrics(ControlSize::Lg).font_size, 14.0);
    }

    #[test]
    fn icon_button_retains_required_name_and_square_geometry() {
        let button = icon::<()>(IconRole::WindowClose, String::from("Close panel"));

        assert_eq!(button.semantic_name_value(), Some("Close panel"));
        assert_eq!(
            button.content.default_width(ControlSize::Sm),
            Length::Fixed(28.0)
        );
        assert_eq!(
            button.content.default_height(ControlSize::Sm),
            Length::Fixed(28.0)
        );
    }

    #[test]
    fn every_density_and_size_uses_form_height_with_invariant_manual_padding() {
        let sizes = [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ];

        for density in ThemeDensity::ALL {
            let theme = ThemeBuilder::new("Button metrics", ThemeMode::Light)
                .density(density)
                .build();
            let _guard = crate::theme::testing::ThemeTestGuard::activate(theme);

            for size in sizes {
                let expected = theme.form_control_metrics(size).height;
                let normal: Element<'_, ()> = secondary("Save").size(size).into();
                let padded: Element<'_, ()> = secondary("Save")
                    .size(size)
                    .padding(Padding::new(40.0))
                    .into();
                let normal = WidgetHarness::new(normal, Size::new(300.0, 80.0));
                let padded = WidgetHarness::new(padded, Size::new(300.0, 80.0));

                assert_eq!(normal.bounds().height, expected);
                assert_eq!(padded.bounds().height, expected);
            }
        }
    }

    #[test]
    fn destructive_solid_uses_the_contrasting_on_danger_focus_treatment() {
        let button = destructive::<()>("Delete");

        assert_eq!(button.focus_ring(), ButtonFocusRing::OnDanger);
    }

    #[test]
    fn loading_preserves_intrinsic_width_and_suppresses_activation() {
        let idle: Element<'_, &'static str> =
            secondary("Save").loading(false).on_press("save").into();
        let loading: Element<'_, &'static str> =
            secondary("Save").loading(true).on_press("save").into();
        let idle = WidgetHarness::new(idle, Size::new(300.0, 80.0));
        let mut loading = WidgetHarness::new(loading, Size::new(300.0, 80.0));

        assert_eq!(idle.bounds(), loading.bounds());
        loading.set_cursor(Point::new(4.0, 4.0));
        assert!(loading
            .update(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left
            )))
            .messages
            .is_empty());
    }

    #[test]
    fn pointer_enter_and_space_activate_while_disabled_does_not() {
        let id = Id::new("button-action");
        let button: Element<'_, &'static str> =
            secondary("Run").id(id.clone()).on_press("run").into();
        let mut harness = WidgetHarness::new(button, Size::new(300.0, 80.0));
        harness.set_cursor(Point::new(4.0, 4.0));
        assert!(harness
            .update(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left
            )))
            .messages
            .is_empty());
        assert_eq!(
            harness
                .update(Event::Mouse(mouse::Event::ButtonReleased(
                    mouse::Button::Left
                )))
                .messages,
            vec!["run"]
        );
        harness.focus(id);
        assert_eq!(
            harness
                .update(key_pressed(Key::Named(Named::Enter)))
                .messages,
            vec!["run"]
        );
        assert_eq!(
            harness
                .update(key_pressed(Key::Named(Named::Space)))
                .messages,
            vec!["run"]
        );

        let disabled: Element<'_, &'static str> =
            secondary("Run").disabled(true).on_press("run").into();
        let mut disabled = WidgetHarness::new(disabled, Size::new(300.0, 80.0));
        disabled.set_cursor(Point::new(4.0, 4.0));
        assert!(disabled
            .update(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left
            )))
            .messages
            .is_empty());
    }

    #[test]
    fn named_fill_button_reflows_wide_narrow_wide_at_invariant_height() {
        let button: Element<'_, &'static str> = named_probe(
            "button",
            secondary("A complete long action label")
                .fill_width()
                .on_press("press"),
        );
        let mut harness = WidgetHarness::new(button, Size::new(320.0, 80.0));
        assert_eq!(
            harness.named_bounds("button").expect("wide").size(),
            Size::new(320.0, 28.0)
        );

        harness.relayout(Size::new(96.0, 80.0));
        assert_eq!(
            harness.named_bounds("button").expect("narrow").size(),
            Size::new(96.0, 28.0)
        );

        harness.relayout(Size::new(320.0, 80.0));
        assert_eq!(
            harness.named_bounds("button").expect("wide again").size(),
            Size::new(320.0, 28.0)
        );
    }

    #[test]
    fn fixed_long_label_reflows_without_changing_outer_height() {
        let button: Element<'_, &'static str> = named_probe(
            "fixed-button",
            secondary("A complete label retained for conditional disclosure")
                .width(Length::Fixed(140.0))
                .on_press("press"),
        );
        let mut harness = WidgetHarness::new(button, Size::new(320.0, 80.0));
        assert_eq!(
            harness.named_bounds("fixed-button").expect("wide").size(),
            Size::new(140.0, 28.0)
        );

        harness.relayout(Size::new(72.0, 80.0));
        assert_eq!(
            harness.named_bounds("fixed-button").expect("narrow").size(),
            Size::new(72.0, 28.0)
        );

        harness.relayout(Size::new(320.0, 80.0));
        assert_eq!(
            harness
                .named_bounds("fixed-button")
                .expect("wide again")
                .size(),
            Size::new(140.0, 28.0)
        );
    }

    #[test]
    fn oversized_custom_content_is_contained_by_fixed_button_geometry() {
        let content: Element<'_, &'static str> = iced::widget::container(
            iced::widget::Space::new()
                .width(Length::Fixed(400.0))
                .height(Length::Fixed(100.0)),
        )
        .into();
        let button: Element<'_, &'static str> = Button::custom(content)
            .width(Length::Fixed(80.0))
            .on_press("press")
            .into();
        let harness = WidgetHarness::new(button, Size::new(300.0, 200.0));

        assert_eq!(harness.bounds().size(), Size::new(80.0, 28.0));
    }

    #[test]
    fn loading_and_disabled_buttons_are_not_focusable_or_keyboard_activatable() {
        for button in [
            secondary("Run").loading(true).on_press("run"),
            secondary("Run").disabled(true).on_press("run"),
        ] {
            let id = Id::unique();
            let element: Element<'_, &'static str> = button.id(id.clone()).into();
            let mut harness = WidgetHarness::new(element, Size::new(200.0, 80.0));
            harness.focus(id);

            assert_eq!(harness.focused_widgets(), 0);
            assert!(harness
                .update(key_pressed(Key::Named(Named::Enter)))
                .messages
                .is_empty());
            assert!(harness
                .update(key_pressed(Key::Named(Named::Space)))
                .messages
                .is_empty());
        }
    }
}

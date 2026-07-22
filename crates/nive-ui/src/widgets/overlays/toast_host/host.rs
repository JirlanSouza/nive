use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use iced::{
    alignment,
    widget::{column, container, mouse_area, row, stack, text},
    Alignment, Length, Padding,
};
use nive_core::{ToastPresentation, ToastTone};

use super::{
    FocusWithinArea, ToastHost, ToastInsets, ToastPosition, ToastStack, BODY_LINE_HEIGHT,
    BODY_SIZE, CARD_GAP, CARD_MAX_WIDTH, CARD_PADDING, ICON_SIZE, MAX_BODY_LINES, MAX_VISIBLE,
    NARROW_CLEARANCE, STACK_GAP, TITLE_SIZE,
};
use crate::theme::{
    self, surface as theme_surface, ShapeSize, SurfaceRole, TextRole, ToneRole, TypographyRole,
};
use crate::widgets::{controls::button, primitives::icon, IconRole};
use crate::Element;

impl ToastInsets {
    pub const NONE: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };
}

impl<'a, Message> ToastHost<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            items: Vec::new(),
            position: ToastPosition::default(),
            insets: ToastInsets::NONE,
            on_pause: None,
            on_resume: None,
            on_focus_enter: None,
            on_focus_exit: None,
        }
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    /// Sets host-provided safe insets the stack must stay clear of.
    pub fn safe_insets(mut self, insets: ToastInsets) -> Self {
        self.insets = insets;
        self
    }

    pub fn on_hover(mut self, pause: Message, resume: Message) -> Self {
        self.on_pause = Some(pause);
        self.on_resume = Some(resume);
        self
    }

    /// Pauses expiry while keyboard focus sits inside the toast stack (a
    /// Tab-focused dismiss or action button), mirroring `on_hover` for
    /// pointer users so a toast never disappears out from under someone
    /// mid-interaction.
    pub fn on_focus_within(mut self, enter: Message, exit: Message) -> Self {
        self.on_focus_enter = Some(enter);
        self.on_focus_exit = Some(exit);
        self
    }

    /// Supplies the toasts to render.
    ///
    /// `on_action` extracts an optional `(label, Message)` secondary action
    /// from the concrete item type; the neutral [`ToastPresentation`]
    /// contract itself never carries an application `Message` (the action
    /// lives on the runtime-owned typed toast, not on the core contract).
    pub fn toasts<T>(
        mut self,
        toasts: impl IntoIterator<Item = &'a T>,
        on_dismiss: impl Fn(T::Id) -> Message + Copy + 'a,
        on_action: impl Fn(&'a T) -> Option<(&'a str, Message)> + Copy + 'a,
    ) -> Self
    where
        T: ToastPresentation + 'a,
        T::Id: Hash,
    {
        self.items = toasts
            .into_iter()
            .take(MAX_VISIBLE)
            .map(|toast| {
                (
                    hash_toast_id(toast.id()),
                    toast_view(toast, on_dismiss, on_action(toast)),
                )
            })
            .collect();
        self
    }

    pub fn into_element(mut self) -> Element<'a, Message> {
        if self.items.is_empty() {
            return self.content;
        }

        // Newest item is always first in `self.items` (nearest the origin).
        // For a bottom origin the newest must land closest to the bottom
        // edge, so the visual column is built oldest-first; for a top
        // origin the given (newest-first) order already reads top-down.
        if is_bottom(self.position) {
            self.items.reverse();
        }

        // Keyed by toast id so a dismissal immediately backfilled by
        // promotion — the common case, since it leaves the visible count
        // unchanged — can't hand the vacated slot's live widget state (an
        // open Tooltip, mid-press interaction) to whatever different toast
        // now occupies it; a positionally diffed `column` would. This uses a
        // local `ToastStack`, not `iced::widget::keyed_column`: that widget's
        // own `diff()` only consults keys when the child count changes,
        // silently falling back to positional diffing for a same-length
        // swap — exactly the case a dismiss-then-promote always produces.
        let toast_stack = ToastStack::new(self.items)
            .spacing(STACK_GAP)
            .width(Length::Fill)
            .max_width(CARD_MAX_WIDTH);
        let toast_stack: Element<'a, Message> = match (self.on_focus_enter, self.on_focus_exit) {
            (Some(enter), Some(exit)) => FocusWithinArea::new(toast_stack)
                .on_focus_within(enter, exit)
                .into(),
            _ => toast_stack.into(),
        };
        let toast_stack: Element<'a, Message> = match (self.on_pause, self.on_resume) {
            (Some(pause), Some(resume)) => mouse_area(toast_stack)
                .on_enter(pause)
                .on_exit(resume)
                .into(),
            _ => toast_stack,
        };
        let (horizontal, vertical) = alignment_for(self.position);
        let overlay = container(toast_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(horizontal)
            .align_y(vertical)
            .padding(overlay_padding(self.insets));

        stack(vec![self.content, overlay.into()])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

pub(super) fn hash_toast_id<Id: Hash>(id: Id) -> u64 {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn overlay_padding(insets: ToastInsets) -> Padding {
    Padding {
        top: insets.top.max(NARROW_CLEARANCE),
        right: insets.right.max(NARROW_CLEARANCE),
        bottom: insets.bottom.max(NARROW_CLEARANCE),
        left: insets.left.max(NARROW_CLEARANCE),
    }
}

pub(super) fn toast_view<'a, Message, T>(
    toast: &'a T,
    on_dismiss: impl Fn(T::Id) -> Message + Copy + 'a,
    action: Option<(&'a str, Message)>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    T: ToastPresentation + 'a,
{
    let tone_role = toast_tone_role(toast.tone());
    let icon_color = theme::active().tone(tone_role).color;
    let tone_icon = icon::role(icon_role_for(toast.tone()))
        .custom_size(ICON_SIZE)
        .color(icon_color);

    let title = text(toast.title())
        .size(TITLE_SIZE)
        .font(theme::typography(TypographyRole::BodyStrong).font)
        .style(theme::text::style(TextRole::Primary))
        .shaping(text::Shaping::Auto);

    let mut text_column = column![title].spacing(CARD_GAP / 2.0);

    if let Some(body) = toast.body() {
        let body_max_height = BODY_SIZE * BODY_LINE_HEIGHT * MAX_BODY_LINES;
        text_column = text_column.push(
            container(
                text(body)
                    .size(BODY_SIZE)
                    .line_height(text::LineHeight::Relative(BODY_LINE_HEIGHT))
                    .style(theme::text::style(TextRole::Secondary))
                    .shaping(text::Shaping::Auto),
            )
            .max_height(body_max_height)
            .clip(true),
        );
    }

    let close = button::icon(IconRole::WindowClose, "Dismiss notification")
        .sm()
        .tooltip("Dismiss")
        .on_press(on_dismiss(toast.id()));

    let mut controls = row![].spacing(CARD_GAP);
    if let Some((label, message)) = action {
        controls = controls.push(button::ghost(label).sm().on_press(message));
    }
    controls = controls.push(close);

    let content = row![tone_icon, text_column.width(Length::Fill), controls]
        .spacing(CARD_GAP)
        .align_y(Alignment::Start)
        .width(Length::Fill);

    container(content)
        .style(theme_surface::style_with_radius(
            SurfaceRole::Elevated,
            theme::active().shape(ShapeSize::Lg).radius(),
        ))
        .padding(Padding::new(CARD_PADDING))
        .width(Length::Fill)
        .max_width(CARD_MAX_WIDTH)
        .into()
}

pub(super) fn toast_tone_role(tone: ToastTone) -> ToneRole {
    match tone {
        ToastTone::Info => ToneRole::Info,
        ToastTone::Success => ToneRole::Success,
        ToastTone::Warning => ToneRole::Warning,
        ToastTone::Danger => ToneRole::Danger,
    }
}

pub(super) fn icon_role_for(tone: ToastTone) -> IconRole {
    match tone {
        ToastTone::Info => IconRole::DialogInformation,
        ToastTone::Success => IconRole::DialogSuccess,
        ToastTone::Warning => IconRole::DialogWarning,
        ToastTone::Danger => IconRole::DialogError,
    }
}

pub(super) fn is_bottom(position: ToastPosition) -> bool {
    matches!(
        position,
        ToastPosition::BottomStart | ToastPosition::BottomEnd
    )
}

pub(super) fn alignment_for(
    position: ToastPosition,
) -> (alignment::Horizontal, alignment::Vertical) {
    match position {
        ToastPosition::TopStart => (alignment::Horizontal::Left, alignment::Vertical::Top),
        ToastPosition::TopEnd => (alignment::Horizontal::Right, alignment::Vertical::Top),
        ToastPosition::BottomStart => (alignment::Horizontal::Left, alignment::Vertical::Bottom),
        ToastPosition::BottomEnd => (alignment::Horizontal::Right, alignment::Vertical::Bottom),
    }
}

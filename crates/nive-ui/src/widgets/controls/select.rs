use iced::{
    overlay::menu,
    widget::{container, pick_list, text},
    Background, Border, Length, Padding, Shadow,
};

use crate::theme::{
    self, control_metrics, text as theme_text, ControlRole, ControlSize, ControlState, SurfaceRole,
    TextRole,
};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
struct SelectMetrics {
    font_size: f32,
    padding_v: f32,
    padding_h: f32,
    radius: f32,
}

pub struct Select<'a, T, Message>
where
    T: ToString + PartialEq + Clone,
{
    options: Vec<T>,
    selected: Option<T>,
    placeholder: Option<&'a str>,
    size: ControlSize,
    width: Length,
    disabled: bool,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
}

impl<'a, T, Message> Select<'a, T, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    Message: Clone + 'a,
{
    pub fn new(options: impl Into<Vec<T>>, selected: Option<T>) -> Self {
        Self {
            options: options.into(),
            selected,
            placeholder: None,
            size: ControlSize::Sm,
            width: Length::Fill,
            disabled: false,
            on_select: None,
            on_open: None,
            on_close: None,
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, on_select: impl Fn(T) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(on_select));
        self
    }

    pub fn on_select_maybe(mut self, on_select: Option<impl Fn(T) -> Message + 'a>) -> Self {
        self.on_select = on_select.map(|on_select| Box::new(on_select) as _);
        self
    }

    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics(self.size);
        let width = self.width;

        if self.disabled || self.on_select.is_none() {
            return disabled_select(
                self.selected
                    .as_ref()
                    .map(ToString::to_string)
                    .or_else(|| self.placeholder.map(str::to_owned))
                    .unwrap_or_default(),
                self.selected.is_none(),
                width,
                metrics,
            );
        }

        let mut select = pick_list(
            self.options,
            self.selected,
            self.on_select.expect("checked above"),
        )
        .padding(
            Padding::ZERO
                .vertical(metrics.padding_v)
                .horizontal(metrics.padding_h),
        )
        .text_size(metrics.font_size)
        .text_shaping(text::Shaping::Auto)
        .width(width)
        .style(style(metrics.radius))
        .menu_style(menu_style(metrics.radius));

        if let Some(placeholder) = self.placeholder {
            select = select.placeholder(placeholder);
        }

        if let Some(message) = self.on_open {
            select = select.on_open(message);
        }

        if let Some(message) = self.on_close {
            select = select.on_close(message);
        }

        select.into()
    }
}

impl<'a, T, Message> From<Select<'a, T, Message>> for Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    Message: Clone + 'a,
{
    fn from(select: Select<'a, T, Message>) -> Self {
        select.into_element()
    }
}

fn disabled_select<'a, Message>(
    label: String,
    is_placeholder: bool,
    width: Length,
    metrics: SelectMetrics,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let label = if is_placeholder && label.is_empty() {
        "Select".to_owned()
    } else {
        label
    };
    let text = if is_placeholder {
        text(label)
            .size(metrics.font_size)
            .shaping(text::Shaping::Auto)
            .style(theme_text::style(TextRole::Muted))
    } else {
        text(label)
            .size(metrics.font_size)
            .shaping(text::Shaping::Auto)
    };

    container(text)
        .style(disabled_style(metrics.radius))
        .padding(
            Padding::ZERO
                .vertical(metrics.padding_v)
                .horizontal(metrics.padding_h),
        )
        .width(width)
        .into()
}

fn metrics(size: ControlSize) -> SelectMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    SelectMetrics {
        font_size: control.font_size,
        padding_v: match size {
            ControlSize::Xs => spacing.xxs,
            ControlSize::Sm => spacing.xs,
            ControlSize::Md => spacing.xs + 1.0,
            ControlSize::Lg => spacing.md,
        },
        padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.lg,
        },
        radius: control.radius,
    }
}

fn style(radius: f32) -> impl Fn(&crate::theme::Theme, pick_list::Status) -> pick_list::Style {
    move |theme: &crate::theme::Theme, status: pick_list::Status| {
        let theme = *theme;
        let state = match status {
            pick_list::Status::Active => ControlState::ENABLED,
            pick_list::Status::Hovered => ControlState::HOVERED,
            pick_list::Status::Opened { .. } => ControlState::FOCUSED,
        };
        let control = theme.control(ControlRole::Standard, state);

        pick_list::Style {
            text_color: control.foreground,
            placeholder_color: theme.text(TextRole::Muted).color,
            handle_color: theme.text(TextRole::Secondary).color,
            background: Background::Color(control.background),
            border: Border {
                color: control.border.color,
                width: control.border.width,
                radius: radius.into(),
            },
        }
    }
}

fn menu_style(radius: f32) -> impl Fn(&crate::theme::Theme) -> menu::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;
        let surface = theme.surface(SurfaceRole::Popover);
        let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        menu::Style {
            background: Background::Color(surface.background),
            border: Border {
                color: surface.border.color,
                width: surface.border.width,
                radius: radius.into(),
            },
            text_color: theme.text(TextRole::Primary).color,
            selected_text_color: selected.foreground,
            selected_background: Background::Color(selected.background),
            shadow: Shadow::default(),
        }
    }
}

fn disabled_style(radius: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, ControlState::DISABLED);

        container::Style {
            text_color: Some(control.foreground),
            background: Some(Background::Color(control.background)),
            border: Border {
                color: control.border.color,
                width: control.border.width,
                radius: radius.into(),
            },
            shadow: Default::default(),
            ..container::Style::default()
        }
    }
}

#[cfg(test)]
mod select_tests {
    use super::*;
    use crate::theme::{BorderRole, Theme};
    use iced::Color;

    #[test]
    fn active_select_uses_app_active_control_background() {
        let theme = Theme::Dark;
        let style = style(6.0)(&theme, pick_list::Status::Active);

        assert_eq!(
            background_color(style.background),
            theme
                .control(ControlRole::Standard, ControlState::ENABLED)
                .background
        );
    }

    #[test]
    fn opened_select_uses_app_focus_border() {
        let theme = Theme::Dark;
        let style = style(6.0)(&theme, pick_list::Status::Opened { is_hovered: false });

        assert_eq!(style.border.color, theme.border(BorderRole::Focus).color);
    }

    #[test]
    fn layout_builders_set_select_width() {
        let default = Select::<_, ()>::new(vec!["Free"], None::<&str>);
        let shrunk = Select::<_, ()>::new(vec!["Free"], None::<&str>).shrink_width();
        let filled = Select::<_, ()>::new(vec!["Free"], None::<&str>).fill_width();

        assert_eq!(default.width, Length::Fill);
        assert_eq!(shrunk.width, Length::Shrink);
        assert_eq!(filled.width, Length::Fill);
    }

    fn background_color(background: Background) -> Color {
        match background {
            Background::Color(color) => color,
            _ => panic!("Expected color background"),
        }
    }
}

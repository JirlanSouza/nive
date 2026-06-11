use iced::{
    overlay::menu,
    widget::{
        button, checkbox, container, pick_list, progress_bar, rule, scrollable, svg, text,
        text_input, toggler,
    },
    Background, Border, Color, Shadow,
};

use super::shape::ShapeRole;
use super::{
    BorderRole, BorderSpec, ControlRole, ControlState, InteractionState, SurfaceRole, TextRole,
    Theme, ToneRole,
};

pub enum ButtonClass<'a> {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
    Embedded,
    Custom(button::StyleFn<'a, Theme>),
}

pub enum CheckboxClass<'a> {
    Standard,
    Custom(checkbox::StyleFn<'a, Theme>),
}

pub enum ContainerClass<'a> {
    Transparent,
    Custom(container::StyleFn<'a, Theme>),
}

pub enum TextClass<'a> {
    Default,
    Role(TextRole),
    Tone(ToneRole),
    OnTone(ToneRole),
    Custom(text::StyleFn<'a, Theme>),
}

pub enum TextInputClass<'a> {
    Standard { validation: FieldValidation },
    Embedded { validation: FieldValidation },
    Custom(text_input::StyleFn<'a, Theme>),
}

pub enum TogglerClass<'a> {
    Standard,
    Custom(toggler::StyleFn<'a, Theme>),
}

pub enum RuleClass<'a> {
    Default,
    Custom(rule::StyleFn<'a, Theme>),
}

pub enum ProgressBarClass<'a> {
    Default,
    Custom(progress_bar::StyleFn<'a, Theme>),
}

pub enum ScrollableClass<'a> {
    Default,
    Custom(scrollable::StyleFn<'a, Theme>),
}

pub enum MenuClass<'a> {
    Default,
    Custom(menu::StyleFn<'a, Theme>),
}

pub enum PickListClass<'a> {
    Default,
    Custom(pick_list::StyleFn<'a, Theme>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldValidation {
    #[default]
    Valid,
    Invalid,
}

impl<'a> Default for ButtonClass<'a> {
    fn default() -> Self {
        Self::Primary
    }
}

impl<'a> Default for CheckboxClass<'a> {
    fn default() -> Self {
        Self::Standard
    }
}

impl<'a> Default for ContainerClass<'a> {
    fn default() -> Self {
        Self::Transparent
    }
}

impl<'a> Default for TextClass<'a> {
    fn default() -> Self {
        Self::Default
    }
}

impl<'a> Default for TextInputClass<'a> {
    fn default() -> Self {
        Self::Standard {
            validation: FieldValidation::Valid,
        }
    }
}

impl<'a> Default for TogglerClass<'a> {
    fn default() -> Self {
        Self::Standard
    }
}

impl<'a> Default for RuleClass<'a> {
    fn default() -> Self {
        Self::Default
    }
}

impl<'a> Default for ProgressBarClass<'a> {
    fn default() -> Self {
        Self::Default
    }
}

impl<'a> Default for ScrollableClass<'a> {
    fn default() -> Self {
        Self::Default
    }
}

impl<'a> Default for MenuClass<'a> {
    fn default() -> Self {
        Self::Default
    }
}

impl<'a> Default for PickListClass<'a> {
    fn default() -> Self {
        Self::Default
    }
}

impl<'a> From<button::StyleFn<'a, Theme>> for ButtonClass<'a> {
    fn from(style: button::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<checkbox::StyleFn<'a, Theme>> for CheckboxClass<'a> {
    fn from(style: checkbox::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<container::StyleFn<'a, Theme>> for ContainerClass<'a> {
    fn from(style: container::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<text::StyleFn<'a, Theme>> for TextClass<'a> {
    fn from(style: text::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<text_input::StyleFn<'a, Theme>> for TextInputClass<'a> {
    fn from(style: text_input::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<toggler::StyleFn<'a, Theme>> for TogglerClass<'a> {
    fn from(style: toggler::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<rule::StyleFn<'a, Theme>> for RuleClass<'a> {
    fn from(style: rule::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<progress_bar::StyleFn<'a, Theme>> for ProgressBarClass<'a> {
    fn from(style: progress_bar::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<scrollable::StyleFn<'a, Theme>> for ScrollableClass<'a> {
    fn from(style: scrollable::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<menu::StyleFn<'a, Theme>> for MenuClass<'a> {
    fn from(style: menu::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<pick_list::StyleFn<'a, Theme>> for PickListClass<'a> {
    fn from(style: pick_list::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl button::Catalog for Theme {
    type Class<'a> = ButtonClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        ButtonClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        match class {
            ButtonClass::Primary => primary_button(self, status),
            ButtonClass::Secondary => secondary_button(self, status),
            ButtonClass::Outline => outline_button(self, status),
            ButtonClass::Ghost => ghost_button(self, status),
            ButtonClass::Destructive => destructive_button(self, status),
            ButtonClass::Link => link_button(self, status),
            ButtonClass::Embedded => embedded_button(self, status),
            ButtonClass::Custom(style) => style(self, status),
        }
    }
}

impl checkbox::Catalog for Theme {
    type Class<'a> = CheckboxClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        CheckboxClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: checkbox::Status) -> checkbox::Style {
        match class {
            CheckboxClass::Standard => default_checkbox(self, status),
            CheckboxClass::Custom(style) => style(self, status),
        }
    }
}

impl container::Catalog for Theme {
    type Class<'a> = ContainerClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        ContainerClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        match class {
            ContainerClass::Transparent => container::Style::default(),
            ContainerClass::Custom(style) => style(self),
        }
    }
}

impl text_input::Catalog for Theme {
    type Class<'a> = TextInputClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        TextInputClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: text_input::Status) -> text_input::Style {
        match class {
            TextInputClass::Standard { validation } => {
                standard_text_input(self, *validation, status)
            }
            TextInputClass::Embedded { validation } => {
                embedded_text_input(self, *validation, status)
            }
            TextInputClass::Custom(style) => style(self, status),
        }
    }
}

impl toggler::Catalog for Theme {
    type Class<'a> = TogglerClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        TogglerClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: toggler::Status) -> toggler::Style {
        match class {
            TogglerClass::Standard => default_toggler(self, status),
            TogglerClass::Custom(style) => style(self, status),
        }
    }
}

impl text::Catalog for Theme {
    type Class<'a> = TextClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        TextClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> text::Style {
        match class {
            TextClass::Default => text::Style::default(),
            TextClass::Role(role) => text::Style {
                color: Some(self.text(*role).color),
            },
            TextClass::Tone(role) => text::Style {
                color: Some(self.tone(*role).color),
            },
            TextClass::OnTone(role) => text::Style {
                color: Some(self.tone(*role).on_color),
            },
            TextClass::Custom(style) => style(self),
        }
    }
}

impl svg::Catalog for Theme {
    type Class<'a> = svg::StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme, _status| svg::Style::default())
    }

    fn style(&self, class: &Self::Class<'_>, status: svg::Status) -> svg::Style {
        class(self, status)
    }
}

impl rule::Catalog for Theme {
    type Class<'a> = RuleClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        RuleClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> rule::Style {
        match class {
            RuleClass::Default => default_rule(self),
            RuleClass::Custom(style) => style(self),
        }
    }
}

impl progress_bar::Catalog for Theme {
    type Class<'a> = ProgressBarClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        ProgressBarClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> progress_bar::Style {
        match class {
            ProgressBarClass::Default => default_progress_bar(self),
            ProgressBarClass::Custom(style) => style(self),
        }
    }
}

impl scrollable::Catalog for Theme {
    type Class<'a> = ScrollableClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        ScrollableClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: scrollable::Status) -> scrollable::Style {
        match class {
            ScrollableClass::Default => default_scrollable(self, status),
            ScrollableClass::Custom(style) => style(self, status),
        }
    }
}

impl menu::Catalog for Theme {
    type Class<'a> = MenuClass<'a>;

    fn default<'a>() -> <Self as menu::Catalog>::Class<'a> {
        MenuClass::default()
    }

    fn style(&self, class: &<Self as menu::Catalog>::Class<'_>) -> menu::Style {
        match class {
            MenuClass::Default => default_menu(self),
            MenuClass::Custom(style) => style(self),
        }
    }
}

impl pick_list::Catalog for Theme {
    type Class<'a> = PickListClass<'a>;

    fn default<'a>() -> <Self as pick_list::Catalog>::Class<'a> {
        PickListClass::default()
    }

    fn style(
        &self,
        class: &<Self as pick_list::Catalog>::Class<'_>,
        status: pick_list::Status,
    ) -> pick_list::Style {
        match class {
            PickListClass::Default => default_pick_list(self, status),
            PickListClass::Custom(style) => style(self, status),
        }
    }
}

fn button_control_state(status: button::Status) -> ControlState {
    match status {
        button::Status::Active => ControlState::ENABLED,
        button::Status::Hovered => ControlState::HOVERED,
        button::Status::Pressed => ControlState::PRESSED,
        button::Status::Disabled => ControlState::DISABLED,
    }
}

fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let tone = theme.tone(ToneRole::Primary);
    let alpha = match status {
        button::Status::Active => 1.0,
        button::Status::Hovered => 0.90,
        button::Status::Pressed => 0.82,
        button::Status::Disabled => 0.45,
    };

    button::Style {
        background: Some(Background::Color(tone.color.scale_alpha(alpha))),
        text_color: theme.tone(ToneRole::Primary).on_color.scale_alpha(
            if matches!(status, button::Status::Disabled) {
                0.65
            } else {
                1.0
            },
        ),
        border: transparent_border_with_radius(theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
    let disabled = theme.control(ControlRole::Standard, ControlState::DISABLED);
    let (background, text_color, border) = match status {
        button::Status::Active => (selected.background, selected.foreground, selected.border),
        button::Status::Hovered => (
            selected.background.scale_alpha(1.25),
            selected.foreground,
            selected.border,
        ),
        button::Status::Pressed => (
            selected.background.scale_alpha(0.88),
            selected.foreground,
            selected.border,
        ),
        button::Status::Disabled => (disabled.background, disabled.foreground, disabled.border),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: border_with_radius(border, theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn outline_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Standard, button_control_state(status));
    let background = match status {
        button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
        button::Status::Hovered | button::Status::Pressed => control.background,
    };
    let text_color = if matches!(status, button::Status::Disabled) {
        control.foreground
    } else {
        theme.text(TextRole::Primary).color
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: border_with_radius(control.border, theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn ghost_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Standard, button_control_state(status));
    let background = match status {
        button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
        button::Status::Hovered | button::Status::Pressed => control.background,
    };
    let text_color = match status {
        button::Status::Hovered | button::Status::Pressed => theme.text(TextRole::Primary).color,
        button::Status::Disabled => theme.text(TextRole::Muted).color.scale_alpha(0.55),
        button::Status::Active => theme.text(TextRole::Secondary).color,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: transparent_border_with_radius(theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn destructive_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let tone = theme.tone(ToneRole::Danger);
    let background = match status {
        button::Status::Active => tone.container,
        button::Status::Hovered => tone.container.scale_alpha(1.35),
        button::Status::Pressed => tone.container.scale_alpha(1.12),
        button::Status::Disabled => tone.container.scale_alpha(0.55),
    };
    let text_color = if matches!(status, button::Status::Disabled) {
        tone.color.scale_alpha(0.55)
    } else {
        tone.color
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: border_with_radius(tone.border, theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn link_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let tone = theme.tone(ToneRole::Primary);
    let alpha = match status {
        button::Status::Active => 1.0,
        button::Status::Hovered => 0.88,
        button::Status::Pressed => 0.76,
        button::Status::Disabled => 0.50,
    };

    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: tone.color.scale_alpha(alpha),
        border: transparent_border_with_radius(theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn embedded_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Embedded, button_control_state(status));
    let foreground = match status {
        button::Status::Hovered | button::Status::Pressed => theme.text(TextRole::Primary).color,
        button::Status::Disabled => theme.text(TextRole::Muted).color.scale_alpha(0.55),
        button::Status::Active => theme.text(TextRole::Secondary).color,
    };
    let background = match status {
        button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
        button::Status::Hovered => control.background.scale_alpha(0.70),
        button::Status::Pressed => control.background.scale_alpha(0.86),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: foreground,
        border: transparent_border_with_radius(theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn default_checkbox(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let theme = *theme;
    let is_checked = match status {
        checkbox::Status::Active { is_checked }
        | checkbox::Status::Hovered { is_checked }
        | checkbox::Status::Disabled { is_checked } => is_checked,
    };
    let state = match status {
        checkbox::Status::Active { .. } => ControlState::ENABLED,
        checkbox::Status::Hovered { .. } => ControlState::HOVERED,
        checkbox::Status::Disabled { .. } => ControlState::DISABLED,
    };
    let control = theme.control(ControlRole::Standard, state);
    let tone = theme.tone(ToneRole::Primary);
    let disabled = matches!(status, checkbox::Status::Disabled { .. });
    let alpha = if disabled { 0.55 } else { 1.0 };

    checkbox::Style {
        background: Background::Color(if is_checked {
            tone.color.scale_alpha(alpha)
        } else {
            control.background
        }),
        icon_color: if is_checked {
            theme.tone(ToneRole::Primary).on_color.scale_alpha(alpha)
        } else {
            Color::TRANSPARENT
        },
        border: border_with_radius(
            if is_checked {
                BorderSpec::new(tone.border.color.scale_alpha(alpha), tone.border.width)
            } else {
                control.border
            },
            theme.shape(ShapeRole::Small).radius(),
        ),
        text_color: Some(if disabled {
            theme.text(TextRole::Muted).color.scale_alpha(0.65)
        } else {
            theme.text(TextRole::Secondary).color
        }),
    }
}

fn standard_text_input(
    theme: &Theme,
    validation: FieldValidation,
    status: text_input::Status,
) -> text_input::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Standard, text_input_control_state(status));
    let muted = theme.text(TextRole::Muted).color;
    let disabled = matches!(status, text_input::Status::Disabled);

    let mut style = text_input::Style {
        background: Background::Color(control.background),
        border: border_with_radius(control.border, theme.shape(ShapeRole::Medium).radius()),
        icon: alpha_when_disabled(muted, disabled),
        placeholder: alpha_when_disabled(muted, disabled),
        value: alpha_when_disabled(control.foreground, disabled),
        selection: theme
            .tone(ToneRole::Primary)
            .color
            .scale_alpha(if disabled { 0.15 } else { 0.30 }),
    };

    apply_standard_text_input_validation(&mut style, theme, validation, disabled);

    style
}

fn embedded_text_input(
    theme: &Theme,
    validation: FieldValidation,
    status: text_input::Status,
) -> text_input::Style {
    let theme = *theme;
    let muted = theme.text(TextRole::Muted).color;
    let value = theme.text(TextRole::Primary).color;
    let disabled = matches!(status, text_input::Status::Disabled);
    let mut style = text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: transparent_border_with_radius(theme.shape(ShapeRole::Medium).radius()),
        icon: alpha_when_disabled(muted, disabled),
        placeholder: alpha_when_disabled(muted, disabled),
        value: alpha_when_disabled(value, disabled),
        selection: theme
            .tone(ToneRole::Primary)
            .color
            .scale_alpha(if disabled { 0.15 } else { 0.30 }),
    };

    if matches!(validation, FieldValidation::Invalid) {
        style.selection =
            theme
                .tone(ToneRole::Danger)
                .color
                .scale_alpha(if disabled { 0.1 } else { 0.2 });
    }

    style
}

fn apply_standard_text_input_validation(
    style: &mut text_input::Style,
    theme: Theme,
    validation: FieldValidation,
    disabled: bool,
) {
    if matches!(validation, FieldValidation::Invalid) {
        let danger = theme.border(BorderRole::Danger);

        style.border.color = alpha_when_disabled(danger.color, disabled);
        style.border.width = danger.width;
        style.selection =
            theme
                .tone(ToneRole::Danger)
                .color
                .scale_alpha(if disabled { 0.1 } else { 0.2 });
    }
}

fn default_toggler(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let theme = *theme;
    let is_toggled = match status {
        toggler::Status::Active { is_toggled }
        | toggler::Status::Hovered { is_toggled }
        | toggler::Status::Disabled { is_toggled } => is_toggled,
    };
    let state = match status {
        toggler::Status::Active { .. } => ControlState::ENABLED,
        toggler::Status::Hovered { .. } => ControlState::HOVERED,
        toggler::Status::Disabled { .. } => ControlState::DISABLED,
    };
    let control = theme.control(ControlRole::Standard, state);
    let alpha = if matches!(status, toggler::Status::Disabled { .. }) {
        0.5
    } else {
        1.0
    };
    let background = if is_toggled {
        theme.tone(ToneRole::Primary).color
    } else {
        control.background
    };
    let foreground = if is_toggled {
        theme.tone(ToneRole::Primary).on_color
    } else {
        theme.text(TextRole::Secondary).color
    };
    let border = theme.border(BorderRole::Default);

    toggler::Style {
        background: Background::Color(background.scale_alpha(alpha)),
        foreground: Background::Color(foreground.scale_alpha(alpha)),
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        background_border_width: border.width,
        background_border_color: border.color.scale_alpha(alpha),
        text_color: None,
        border_radius: None,
        padding_ratio: 0.1,
    }
}

fn default_rule(theme: &Theme) -> rule::Style {
    let border = theme.border(BorderRole::Default);

    rule::Style {
        color: border.color,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

fn default_progress_bar(theme: &Theme) -> progress_bar::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Standard, ControlState::ENABLED);
    let tone = theme.tone(ToneRole::Primary);

    progress_bar::Style {
        background: Background::Color(control.background),
        bar: Background::Color(tone.color),
        border: border_with_radius(BorderSpec::none(), theme.shape(ShapeRole::Small).radius()),
    }
}

fn default_scrollable(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let theme = *theme;
    let rail_color = theme.border(BorderRole::Subtle).color.scale_alpha(0.35);
    let scroller_color = theme.text(TextRole::Muted).color.scale_alpha(0.50);
    let active_rail = scrollable::Rail {
        background: Some(Background::Color(rail_color)),
        border: border_with_radius(
            BorderSpec::none(),
            theme.shape(ShapeRole::ExtraSmall).radius(),
        ),
        scroller: scrollable::Scroller {
            background: Background::Color(scroller_color),
            border: border_with_radius(
                BorderSpec::none(),
                theme.shape(ShapeRole::ExtraSmall).radius(),
            ),
        },
    };
    let primary_rail = scrollable::Rail {
        scroller: scrollable::Scroller {
            background: Background::Color(theme.tone(ToneRole::Primary).color),
            ..active_rail.scroller
        },
        ..active_rail
    };
    let popover = theme.surface(SurfaceRole::Popover);
    let auto_scroll = scrollable::AutoScroll {
        background: Background::Color(popover.background.scale_alpha(0.90)),
        border: border_with_radius(popover.border, theme.shape(ShapeRole::ExtraLarge).radius()),
        shadow: popover.shadow,
        icon: theme.text(TextRole::Secondary).color,
    };

    let (vertical_rail, horizontal_rail) = match status {
        scrollable::Status::Active { .. } => (active_rail, active_rail),
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => (
            if is_vertical_scrollbar_hovered {
                primary_rail
            } else {
                active_rail
            },
            if is_horizontal_scrollbar_hovered {
                primary_rail
            } else {
                active_rail
            },
        ),
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => (
            if is_vertical_scrollbar_dragged {
                primary_rail
            } else {
                active_rail
            },
            if is_horizontal_scrollbar_dragged {
                primary_rail
            } else {
                active_rail
            },
        ),
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail,
        horizontal_rail,
        gap: None,
        auto_scroll,
    }
}

fn default_menu(theme: &Theme) -> menu::Style {
    let theme = *theme;
    let surface = theme.surface(SurfaceRole::Popover);
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);

    menu::Style {
        background: Background::Color(surface.background),
        border: border_with_radius(surface.border, theme.shape(ShapeRole::Medium).radius()),
        text_color: surface.foreground,
        selected_text_color: selected.foreground,
        selected_background: Background::Color(selected.background),
        shadow: surface.shadow,
    }
}

fn default_pick_list(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
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
        border: border_with_radius(control.border, theme.shape(ShapeRole::Medium).radius()),
    }
}

fn text_input_control_state(status: text_input::Status) -> ControlState {
    match status {
        text_input::Status::Active => ControlState::ENABLED,
        text_input::Status::Hovered => ControlState::HOVERED,
        text_input::Status::Focused { is_hovered } => {
            ControlState::new().interaction(if is_hovered {
                InteractionState::FOCUSED.hovered()
            } else {
                InteractionState::FOCUSED
            })
        }
        text_input::Status::Disabled => ControlState::DISABLED,
    }
}

fn border_with_radius(spec: BorderSpec, radius: impl Into<iced::border::Radius>) -> Border {
    Border {
        color: spec.color,
        width: spec.width,
        radius: radius.into(),
    }
}

fn transparent_border_with_radius(radius: impl Into<iced::border::Radius>) -> Border {
    border_with_radius(BorderSpec::none(), radius)
}

fn alpha_when_disabled(color: Color, disabled: bool) -> Color {
    if disabled {
        color.scale_alpha(0.5)
    } else {
        color
    }
}

#[cfg(test)]
mod catalog_tests {
    use super::*;

    #[test]
    fn catalog_defaults_are_semantic_classes() {
        assert!(matches!(
            <Theme as button::Catalog>::default(),
            ButtonClass::Primary
        ));
        assert!(matches!(
            <Theme as checkbox::Catalog>::default(),
            CheckboxClass::Standard
        ));
        assert!(matches!(
            <Theme as container::Catalog>::default(),
            ContainerClass::Transparent
        ));
        assert!(matches!(
            <Theme as text::Catalog>::default(),
            TextClass::Default
        ));
        assert!(matches!(
            <Theme as text_input::Catalog>::default(),
            TextInputClass::Standard {
                validation: FieldValidation::Valid
            }
        ));
        assert!(matches!(
            <Theme as toggler::Catalog>::default(),
            TogglerClass::Standard
        ));
        assert!(matches!(
            <Theme as rule::Catalog>::default(),
            RuleClass::Default
        ));
        assert!(matches!(
            <Theme as progress_bar::Catalog>::default(),
            ProgressBarClass::Default
        ));
        assert!(matches!(
            <Theme as scrollable::Catalog>::default(),
            ScrollableClass::Default
        ));
        assert!(matches!(
            <Theme as menu::Catalog>::default(),
            MenuClass::Default
        ));
        assert!(matches!(
            <Theme as pick_list::Catalog>::default(),
            PickListClass::Default
        ));
    }

    #[test]
    fn default_text_inherits_container_color() {
        let class = <Theme as text::Catalog>::default();
        let style = <Theme as text::Catalog>::style(&Theme::Dark, &class);

        assert_eq!(style, text::Style::default());
    }

    #[test]
    fn default_svg_preserves_asset_color() {
        let class = <Theme as svg::Catalog>::default();
        let style = <Theme as svg::Catalog>::style(&Theme::Dark, &class, svg::Status::Idle);

        assert_eq!(style, svg::Style::default());
    }

    #[test]
    fn default_container_is_transparent() {
        let class = <Theme as container::Catalog>::default();
        let style = <Theme as container::Catalog>::style(&Theme::Dark, &class);

        assert_eq!(style, container::Style::default());
    }

    #[test]
    fn button_primary_class_uses_semantic_primary() {
        let theme = Theme::Dark;
        let class = ButtonClass::Primary;
        let style = <Theme as button::Catalog>::style(&theme, &class, button::Status::Active);

        assert_eq!(
            background_color(style.background),
            theme.tone(ToneRole::Primary).color
        );
        assert_eq!(style.text_color, theme.tone(ToneRole::Primary).on_color);
    }

    #[test]
    fn button_destructive_class_uses_danger_tone() {
        let theme = Theme::Dark;
        let class = ButtonClass::Destructive;
        let style = <Theme as button::Catalog>::style(&theme, &class, button::Status::Active);
        let danger = theme.tone(ToneRole::Danger);

        assert_eq!(background_color(style.background), danger.container);
        assert_eq!(style.text_color, danger.color);
        assert_eq!(style.border.color, danger.border.color);
        assert_eq!(style.border.width, danger.border.width);
    }

    #[test]
    fn button_link_class_uses_primary_text_without_chrome() {
        let theme = Theme::Dark;
        let class = ButtonClass::Link;
        let style = <Theme as button::Catalog>::style(&theme, &class, button::Status::Active);

        assert_eq!(background_color(style.background), Color::TRANSPARENT);
        assert_eq!(style.text_color, theme.tone(ToneRole::Primary).color);
        assert_eq!(style.border.color, Color::TRANSPARENT);
        assert_eq!(style.border.width, 0.0);
    }

    #[test]
    fn default_button_uses_semantic_primary() {
        let theme = Theme::Dark;
        let class = <Theme as button::Catalog>::default();
        let style = <Theme as button::Catalog>::style(&theme, &class, button::Status::Active);

        assert_eq!(
            background_color(style.background),
            theme.tone(ToneRole::Primary).color
        );
        assert_eq!(style.text_color, theme.tone(ToneRole::Primary).on_color);
    }

    #[test]
    fn default_checkbox_uses_semantic_primary_when_checked() {
        let theme = Theme::Dark;
        let class = <Theme as checkbox::Catalog>::default();
        let style = <Theme as checkbox::Catalog>::style(
            &theme,
            &class,
            checkbox::Status::Active { is_checked: true },
        );

        assert_eq!(
            background_color(Some(style.background)),
            theme.tone(ToneRole::Primary).color
        );
        assert_eq!(style.icon_color, theme.tone(ToneRole::Primary).on_color);
    }

    #[test]
    fn default_checkbox_uses_active_control_when_unchecked() {
        let theme = Theme::Dark;
        let class = <Theme as checkbox::Catalog>::default();
        let style = <Theme as checkbox::Catalog>::style(
            &theme,
            &class,
            checkbox::Status::Active { is_checked: false },
        );

        assert_eq!(
            background_color(Some(style.background)),
            theme
                .control(ControlRole::Standard, ControlState::ENABLED)
                .background
        );
    }

    #[test]
    fn default_text_input_uses_focused_control_border() {
        let theme = Theme::Dark;
        let class = <Theme as text_input::Catalog>::default();
        let style = <Theme as text_input::Catalog>::style(
            &theme,
            &class,
            text_input::Status::Focused { is_hovered: false },
        );
        let control = theme.control(ControlRole::Standard, ControlState::FOCUSED);

        assert_eq!(style.border.color, control.border.color);
        assert_eq!(style.value, control.foreground);
    }

    #[test]
    fn invalid_standard_text_input_uses_danger_border() {
        let theme = Theme::Dark;
        let class = TextInputClass::Standard {
            validation: FieldValidation::Invalid,
        };
        let style =
            <Theme as text_input::Catalog>::style(&theme, &class, text_input::Status::Active);
        let danger = theme.border(BorderRole::Danger);

        assert_eq!(style.border.color, danger.color);
        assert_eq!(style.border.width, danger.width);
        assert_eq!(
            style.selection,
            theme.tone(ToneRole::Danger).color.scale_alpha(0.2)
        );
    }

    #[test]
    fn default_toggler_uses_semantic_primary_when_toggled() {
        let theme = Theme::Dark;
        let class = <Theme as toggler::Catalog>::default();
        let style = <Theme as toggler::Catalog>::style(
            &theme,
            &class,
            toggler::Status::Active { is_toggled: true },
        );

        assert_eq!(
            background_color(Some(style.background)),
            theme.tone(ToneRole::Primary).color
        );
    }

    #[test]
    fn default_rule_uses_default_border_role() {
        let theme = Theme::Dark;
        let class = <Theme as rule::Catalog>::default();
        let style = <Theme as rule::Catalog>::style(&theme, &class);

        assert_eq!(style.color, theme.border(BorderRole::Default).color);
    }

    #[test]
    fn default_progress_bar_uses_semantic_primary_bar() {
        let theme = Theme::Dark;
        let class = <Theme as progress_bar::Catalog>::default();
        let style = <Theme as progress_bar::Catalog>::style(&theme, &class);

        assert_eq!(
            background_color(Some(style.bar)),
            theme.tone(ToneRole::Primary).color
        );
    }

    #[test]
    fn default_scrollable_hover_uses_primary_scroller() {
        let theme = Theme::Dark;
        let class = <Theme as scrollable::Catalog>::default();
        let style = <Theme as scrollable::Catalog>::style(
            &theme,
            &class,
            scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: false,
                is_vertical_scrollbar_hovered: true,
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );

        assert_eq!(
            background_color(Some(style.vertical_rail.scroller.background)),
            theme.tone(ToneRole::Primary).color
        );
    }

    #[test]
    fn default_menu_uses_popover_surface() {
        let theme = Theme::Dark;
        let class = <Theme as menu::Catalog>::default();
        let style = <Theme as menu::Catalog>::style(&theme, &class);
        let surface = theme.surface(SurfaceRole::Popover);

        assert_eq!(background_color(Some(style.background)), surface.background);
        assert_eq!(style.text_color, surface.foreground);
        assert_eq!(style.shadow, surface.shadow);
    }

    #[test]
    fn default_pick_list_opened_uses_focused_control() {
        let theme = Theme::Dark;
        let class = <Theme as pick_list::Catalog>::default();
        let style = <Theme as pick_list::Catalog>::style(
            &theme,
            &class,
            pick_list::Status::Opened { is_hovered: false },
        );
        let control = theme.control(ControlRole::Standard, ControlState::FOCUSED);

        assert_eq!(background_color(Some(style.background)), control.background);
        assert_eq!(style.border.color, control.border.color);
    }

    fn background_color(background: Option<Background>) -> Color {
        match background {
            Some(Background::Color(color)) => color,
            _ => panic!("Expected color background"),
        }
    }
}

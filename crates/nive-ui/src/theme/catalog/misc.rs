use iced::{
    overlay::menu,
    widget::{container, pick_list, progress_bar, rule, scrollable},
    Background,
};

use super::shared::border_with_radius;
use crate::theme::{
    BorderRole, BorderSpec, ControlRole, ControlState, ShapeSize, SurfaceRole, TextRole, Theme,
    ToneRole,
};

pub(super) fn default_rule(theme: &Theme) -> rule::Style {
    let border = theme.border(BorderRole::Default);

    rule::Style {
        color: border.color,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

pub(super) fn default_progress_bar(theme: &Theme) -> progress_bar::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Standard, ControlState::ENABLED);
    let tone = theme.tone(ToneRole::Accent);

    progress_bar::Style {
        background: Background::Color(control.background),
        bar: Background::Color(tone.color),
        border: border_with_radius(BorderSpec::none(), theme.shape(ShapeSize::Sm).radius()),
    }
}

pub(super) fn default_scrollable(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let theme = *theme;
    let rail_color = theme.border(BorderRole::Subtle).color.scale_alpha(0.35);
    let scroller_color = theme.text(TextRole::Muted).color.scale_alpha(0.50);
    let active_rail = scrollable::Rail {
        background: Some(Background::Color(rail_color)),
        border: border_with_radius(BorderSpec::none(), theme.shape(ShapeSize::Xs).radius()),
        scroller: scrollable::Scroller {
            background: Background::Color(scroller_color),
            border: border_with_radius(BorderSpec::none(), theme.shape(ShapeSize::Xs).radius()),
        },
    };
    let primary_rail = scrollable::Rail {
        scroller: scrollable::Scroller {
            background: Background::Color(theme.tone(ToneRole::Accent).color),
            ..active_rail.scroller
        },
        ..active_rail
    };
    let popover = theme.surface(SurfaceRole::Popover);
    let auto_scroll = scrollable::AutoScroll {
        background: Background::Color(popover.background.scale_alpha(0.90)),
        border: border_with_radius(popover.border, theme.shape(ShapeSize::Xl).radius()),
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

pub(super) fn default_menu(theme: &Theme) -> menu::Style {
    let theme = *theme;
    let surface = theme.surface(SurfaceRole::Popover);
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);

    menu::Style {
        background: Background::Color(surface.background),
        border: border_with_radius(surface.border, theme.shape(ShapeSize::Md).radius()),
        text_color: surface.foreground,
        selected_text_color: selected.foreground,
        selected_background: Background::Color(selected.background),
        shadow: surface.shadow,
    }
}

pub(super) fn default_pick_list(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
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
        border: border_with_radius(control.border, theme.shape(ShapeSize::Md).radius()),
    }
}

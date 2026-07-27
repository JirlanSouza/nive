use iced::{
    border::Radius,
    widget::{button, container, text},
    Background, Border, Color, Shadow,
};

use crate::advanced::control_style::{
    border_with_radius, disabled_alpha, transparent_border, transparent_border_with_radius,
};

use crate::theme::{
    self, control_metrics, BorderRole, BorderSpec, ControlRole, ControlSize, ControlState,
    InteractionState, TextRole,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeItemVariant {
    Default,
    Selected,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeItemMetrics {
    pub height: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub expander_side: f32,
    pub indent: f32,
    pub gap: f32,
    pub padding_h: f32,
    pub radius: f32,
    pub tone_size: f32,
}

pub fn metrics(size: ControlSize) -> TreeItemMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    TreeItemMetrics {
        height: control.height,
        font_size: control.font_size,
        icon_size: control.icon_size,
        expander_side: control.height,
        indent: match size {
            ControlSize::Xs => spacing.md,
            ControlSize::Sm => spacing.lg,
            ControlSize::Md => spacing.xl,
            ControlSize::Lg => spacing.xl + spacing.xs,
        },
        gap: control.gap,
        padding_h: match size {
            ControlSize::Xs => spacing.xs,
            ControlSize::Sm => spacing.sm,
            ControlSize::Md => spacing.md,
            ControlSize::Lg => spacing.md + spacing.xxs,
        },
        radius: control.radius,
        tone_size: match size {
            ControlSize::Xs => 5.0,
            ControlSize::Sm => 6.0,
            ControlSize::Md | ControlSize::Lg => 7.0,
        },
    }
}

pub fn item_style(
    variant: TreeItemVariant,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: button::Status| {
        let theme = *theme;
        let disabled = theme.control(ControlRole::Standard, ControlState::DISABLED);

        // The row container paints every fill, because it is the only element
        // spanning the whole row; this button covers only the space left after
        // indentation and the expander, so a fill here would stop short of the
        // row's leading edge.
        let background = Color::TRANSPARENT;

        let text_color = match (variant, status) {
            (_, button::Status::Disabled) => disabled.foreground,
            (TreeItemVariant::Selected, _) => theme.text(TextRole::Primary).color,
            (TreeItemVariant::Default, button::Status::Hovered | button::Status::Pressed) => {
                theme.text(TextRole::Primary).color
            }
            (TreeItemVariant::Default, _) => theme.text(TextRole::Primary).color,
        };

        let border_radius = match variant {
            TreeItemVariant::Selected => 0.0,
            TreeItemVariant::Default => radius,
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: border_with_radius(BorderSpec::none(), border_radius),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

pub fn expander_style(
    selected: bool,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: button::Status| {
        let theme = *theme;

        // The row container paints the fill for the whole row, expander
        // included, so this button never paints its own.
        let background = Color::TRANSPARENT;

        let text_color = match (selected, status) {
            (_, button::Status::Disabled) => disabled_alpha(theme.text(TextRole::Muted).color),
            (true, _) => theme.text(TextRole::Primary).color,
            (false, button::Status::Hovered | button::Status::Pressed) => {
                theme.text(TextRole::Primary).color
            }
            (false, button::Status::Active) => theme.text(TextRole::Secondary).color,
        };

        let border_radius = if selected { 0.0 } else { radius };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: transparent_border_with_radius(border_radius),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

pub fn indent_style() -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |_theme: &crate::theme::Theme, _status: button::Status| {
        // As with the expander, the row container owns the fill.
        let background = Color::TRANSPARENT;

        button::Style {
            background: Some(Background::Color(background)),
            text_color: Color::TRANSPARENT,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::new(0.0),
            },
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

/// Everything the row container needs to paint itself.
///
/// Grouped rather than passed as a run of booleans, so no caller can transpose
/// two of them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct TreeRowState {
    pub(super) selected: bool,
    pub(super) disabled: bool,
    pub(super) focused: bool,
    pub(super) hovered: bool,
    pub(super) pressed: bool,
    pub(super) dragging: bool,
    pub(super) drop_into: bool,
}

/// The row container paints every fill, because it is the only element that
/// spans the whole row.
///
/// Indentation and the branch expander are siblings of the button, not children
/// of it, so a fill painted by the button stops short of the row's leading edge.
/// Selection was already painted here; hover and pressed join it, which is why
/// the row needs the pointer state that [`super::pointer::PointerProbe`] carries
/// across.
pub(super) fn row_style(row: TreeRowState) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;

        // Selected×disabled is resolved once, centrally (no widget-local
        // alpha); model focus renders a layout-neutral affordance
        // independent of selection.
        let interaction = if row.pressed {
            InteractionState::PRESSED
        } else if row.hovered {
            InteractionState::HOVERED
        } else if row.focused {
            InteractionState::FOCUSED
        } else {
            InteractionState::NONE
        };
        let mut state = ControlState::new().interaction(interaction);
        if row.selected {
            state = state.selected();
        }
        if row.disabled {
            state = state.disabled();
        }
        // Embedded so an untouched, unselected row adds nothing over the surface
        // hosting the tree, and hover/pressed composite over it.
        let control = theme.control(ControlRole::Embedded, state);

        // Focus is a border here, not a fill: a focused-but-unselected row must
        // stay distinguishable from a hovered one.
        let mut background = if row.selected || row.hovered || row.pressed {
            control.background
        } else {
            Color::TRANSPARENT
        };
        if row.dragging {
            background = background.scale_alpha(0.5);
        }

        // A drag-over `Into` target takes visual priority over the focus
        // border: the two never carry real meaning at once, since pointer
        // drag and keyboard focus-visible are mutually exclusive input modes.
        let border = if row.drop_into {
            border_with_radius(theme.border(BorderRole::Accent), 0.0)
        } else if row.focused {
            border_with_radius(theme.border(BorderRole::Focus), 0.0)
        } else {
            transparent_border()
        };

        container::Style {
            text_color: None,
            background: Some(Background::Color(background)),
            border,
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

/// Style for the thin accent bar marking a Before/After drop edge.
pub fn drop_edge_style() -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;

        container::Style {
            text_color: None,
            background: Some(Background::Color(theme.border(BorderRole::Accent).color)),
            border: transparent_border(),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

pub fn trailing_text_style(disabled: bool) -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;
        let disabled_control = theme.control(ControlRole::Standard, ControlState::DISABLED);

        let color = if disabled {
            disabled_control.foreground
        } else {
            theme.text(TextRole::Secondary).color
        };

        text::Style { color: Some(color) }
    }
}

#[cfg(test)]
mod tree_item_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn selected_item_uses_app_selected_control_background() {
        let theme = Theme::Dark;
        let item = row_style(TreeRowState {
            selected: true,
            ..TreeRowState::default()
        })(&theme);

        assert_eq!(
            background_color(&item),
            theme
                .control(ControlRole::Selectable, ControlState::SELECTED)
                .background
        );
        assert_eq!(item.border.width, 0.0);
        assert_eq!(item.border.radius, Radius::default());
    }

    #[test]
    fn dragging_row_dims_the_resolved_background() {
        let theme = Theme::Dark;
        let plain = row_style(TreeRowState {
            selected: true,
            ..TreeRowState::default()
        })(&theme);
        let dragging = row_style(TreeRowState {
            selected: true,
            dragging: true,
            ..TreeRowState::default()
        })(&theme);

        assert_eq!(
            background_color(&dragging),
            background_color(&plain).scale_alpha(0.5)
        );
    }

    #[test]
    fn drop_into_target_renders_the_accent_border_over_focus() {
        let theme = Theme::Dark;
        let item = row_style(TreeRowState {
            focused: true,
            drop_into: true,
            ..TreeRowState::default()
        })(&theme);

        assert_eq!(item.border.color, theme.border(BorderRole::Accent).color);
        assert!(item.border.width > 0.0);
    }

    #[test]
    fn disabled_selected_row_uses_the_shared_resolver_not_a_local_alpha() {
        let theme = Theme::Dark;
        let item = row_style(TreeRowState {
            selected: true,
            disabled: true,
            ..TreeRowState::default()
        })(&theme);
        let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        // Same canonical dimming button/style.rs and selectable_item.rs use
        // — no widget-local 0.55.
        assert_eq!(
            background_color(&item),
            selected.background.scale_alpha(0.60)
        );
    }

    #[test]
    fn focused_row_renders_the_focus_border_independent_of_selection() {
        let theme = Theme::Dark;
        let unfocused = row_style(TreeRowState::default())(&theme);
        let focused = row_style(TreeRowState {
            focused: true,
            ..TreeRowState::default()
        })(&theme);

        assert_eq!(unfocused.border.width, 0.0);
        assert_eq!(focused.border.color, theme.border(BorderRole::Focus).color);
        assert!(focused.border.width > 0.0);
        // Focus is layout-neutral: it doesn't change the resolved background.
        assert_eq!(background_color(&focused), background_color(&unfocused));
    }

    #[test]
    fn selected_item_button_keeps_row_background_visible() {
        let theme = Theme::Dark;
        let item = item_style(TreeItemVariant::Selected, 6.0)(&theme, button::Status::Active);

        assert_eq!(button_background_color(&item), Color::TRANSPARENT);
        assert_eq!(item.text_color, theme.text(TextRole::Primary).color);
        assert_eq!(item.border.width, 0.0);
        assert_eq!(item.border.radius, Radius::default());
    }

    #[test]
    fn default_item_uses_primary_text() {
        let theme = Theme::Dark;
        let item = item_style(TreeItemVariant::Default, 6.0)(&theme, button::Status::Active);

        assert_eq!(item.text_color, theme.text(TextRole::Primary).color);
    }

    #[test]
    fn indentation_grows_with_size() {
        assert!(metrics(ControlSize::Xs).indent < metrics(ControlSize::Lg).indent);
    }

    #[test]
    fn sm_indentation_uses_compact_theme_spacing() {
        assert_eq!(metrics(ControlSize::Sm).indent, theme::spacing().lg);
    }

    #[test]
    fn only_the_row_paints_a_fill_so_hover_spans_the_whole_row() {
        let theme = Theme::Dark;

        // Indentation and the expander are siblings of the button, so a fill
        // painted by any of the three would stop short of the row's edges. None
        // of them may paint one, in any status.
        for status in [
            button::Status::Active,
            button::Status::Hovered,
            button::Status::Pressed,
            button::Status::Disabled,
        ] {
            assert_eq!(
                button_background_color(&indent_style()(&theme, status)),
                Color::TRANSPARENT,
                "indentation must not paint a fill ({status:?})"
            );
            assert_eq!(
                button_background_color(&expander_style(false, 4.0)(&theme, status)),
                Color::TRANSPARENT,
                "the expander must not paint a fill ({status:?})"
            );
            assert_eq!(
                button_background_color(&item_style(TreeItemVariant::Default, 4.0)(&theme, status)),
                Color::TRANSPARENT,
                "the row button must not paint a fill ({status:?})"
            );
        }

        // The row container, which spans the whole row, is where they land.
        let hovered = row_style(TreeRowState {
            hovered: true,
            ..TreeRowState::default()
        })(&theme);
        let pressed = row_style(TreeRowState {
            pressed: true,
            ..TreeRowState::default()
        })(&theme);
        let untouched = row_style(TreeRowState::default())(&theme);

        assert_eq!(background_color(&untouched), Color::TRANSPARENT);
        assert_ne!(background_color(&hovered).a, 0.0);
        assert!(
            background_color(&pressed).a > background_color(&hovered).a,
            "pressed must intensify past hover"
        );
    }

    #[test]
    fn a_focused_but_unselected_row_stays_distinct_from_a_hovered_one() {
        let theme = Theme::Dark;
        let focused = row_style(TreeRowState {
            focused: true,
            ..TreeRowState::default()
        })(&theme);
        let hovered = row_style(TreeRowState {
            hovered: true,
            ..TreeRowState::default()
        })(&theme);

        // Focus is a border, hover is a fill: the two must not be confusable.
        assert_eq!(background_color(&focused), Color::TRANSPARENT);
        assert_ne!(focused.border.color, Color::TRANSPARENT);
        assert_ne!(background_color(&hovered).a, 0.0);
    }

    fn background_color(style: &container::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }

    fn button_background_color(style: &button::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}

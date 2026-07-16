use iced::{
    overlay::menu,
    widget::{
        button, checkbox, container, pick_list, progress_bar, rule, scrollable, svg, text,
        text_input, toggler,
    },
    Background, Color,
};

use super::{
    button::{destructive_button, ghost_button, outline_button, primary_button, secondary_button},
    ButtonClass, CheckboxClass, ContainerClass, FieldValidation, MenuClass, PickListClass,
    ProgressBarClass, RuleClass, ScrollableClass, TextClass, TextInputClass, TogglerClass,
};
use crate::theme::{BorderRole, ControlRole, ControlState, SurfaceRole, TextRole, Theme, ToneRole};
use crate::widgets::{ButtonIntent, ButtonVariant};

type ButtonStyleFn = fn(&Theme, button::Status) -> button::Style;

#[test]
fn catalog_defaults_are_semantic_classes() {
    assert!(matches!(
        <Theme as button::Catalog>::default(),
        ButtonClass::Standard {
            intent: ButtonIntent::Suggested,
            variant: ButtonVariant::Solid,
        }
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
fn button_suggested_solid_class_uses_accent() {
    let theme = Theme::Dark;
    let class = ButtonClass::Standard {
        intent: ButtonIntent::Suggested,
        variant: ButtonVariant::Solid,
    };
    let style = <Theme as button::Catalog>::style(&theme, &class, button::Status::Active);

    assert_eq!(
        background_color(style.background),
        theme.tone(ToneRole::Accent).color
    );
    assert_eq!(style.text_color, theme.tone(ToneRole::Accent).on_color);
}

#[test]
fn button_destructive_class_uses_danger_tone() {
    let theme = Theme::Dark;
    let class = ButtonClass::Standard {
        intent: ButtonIntent::Destructive,
        variant: ButtonVariant::Solid,
    };
    let style = <Theme as button::Catalog>::style(&theme, &class, button::Status::Active);
    let danger = theme.tone(ToneRole::Danger);

    assert_eq!(background_color(style.background), danger.color);
    assert_eq!(style.text_color, danger.on_color);
    assert_eq!(style.border.color, Color::TRANSPARENT);
    assert_eq!(style.border.width, 0.0);
}

#[test]
fn legacy_button_combinations_match_previous_styles_for_each_status() {
    let theme = Theme::Dark;
    let cases: [(ButtonIntent, ButtonVariant, ButtonStyleFn); 5] = [
        (
            ButtonIntent::Suggested,
            ButtonVariant::Solid,
            primary_button,
        ),
        (
            ButtonIntent::Neutral,
            ButtonVariant::Subtle,
            secondary_button,
        ),
        (
            ButtonIntent::Neutral,
            ButtonVariant::Outline,
            outline_button,
        ),
        (ButtonIntent::Neutral, ButtonVariant::Ghost, ghost_button),
        (
            ButtonIntent::Destructive,
            ButtonVariant::Solid,
            destructive_button,
        ),
    ];
    let statuses = [
        button::Status::Active,
        button::Status::Hovered,
        button::Status::Pressed,
        button::Status::Disabled,
    ];

    for (intent, variant, legacy_style) in cases {
        let class = ButtonClass::Standard { intent, variant };

        for status in statuses {
            assert_eq!(
                <Theme as button::Catalog>::style(&theme, &class, status),
                legacy_style(&theme, status),
                "{intent:?} {variant:?} {status:?}"
            );
        }
    }
}

#[test]
fn default_button_uses_suggested_solid() {
    let theme = Theme::Dark;
    let class = <Theme as button::Catalog>::default();
    let style = <Theme as button::Catalog>::style(&theme, &class, button::Status::Active);

    assert_eq!(
        background_color(style.background),
        theme.tone(ToneRole::Accent).color
    );
    assert_eq!(style.text_color, theme.tone(ToneRole::Accent).on_color);
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
        theme.tone(ToneRole::Accent).color
    );
    assert_eq!(style.icon_color, theme.tone(ToneRole::Accent).on_color);
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
    let style = <Theme as text_input::Catalog>::style(&theme, &class, text_input::Status::Active);
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
        theme.tone(ToneRole::Accent).color
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
        theme.tone(ToneRole::Accent).color
    );
}

#[test]
fn default_scrollable_states_are_transparent_and_axis_independent() {
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

    assert!(style.vertical_rail.background.is_none());
    assert!(style.horizontal_rail.background.is_none());
    assert_eq!(
        background_color(Some(style.vertical_rail.scroller.background)),
        theme.text(TextRole::Secondary).color.scale_alpha(0.78)
    );
    assert_eq!(
        background_color(Some(style.horizontal_rail.scroller.background)),
        theme.text(TextRole::Muted).color.scale_alpha(0.50)
    );

    let dragged = <Theme as scrollable::Catalog>::style(
        &theme,
        &class,
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged: true,
            is_vertical_scrollbar_dragged: false,
            is_horizontal_scrollbar_disabled: false,
            is_vertical_scrollbar_disabled: false,
        },
    );
    assert_eq!(
        background_color(Some(dragged.horizontal_rail.scroller.background)),
        theme.tone(ToneRole::Accent).color
    );
    assert_eq!(
        background_color(Some(dragged.vertical_rail.scroller.background)),
        theme.text(TextRole::Muted).color.scale_alpha(0.50)
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

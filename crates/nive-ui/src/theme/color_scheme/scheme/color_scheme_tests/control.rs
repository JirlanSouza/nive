use super::*;

#[test]
fn focused_control_uses_focus_border() {
    let theme = Theme::Dark;

    assert_eq!(
        theme
            .control(ControlRole::Standard, ControlState::FOCUSED)
            .border,
        theme.border(BorderRole::Focus)
    );
}

#[test]
fn focused_hovered_control_keeps_focus_border_and_hover_background() {
    let theme = Theme::Dark;
    let focused_hovered = ControlState::new().interaction(InteractionState::FOCUSED.hovered());
    let control = theme.control(ControlRole::Standard, focused_hovered);

    assert_eq!(control.border, theme.border(BorderRole::Focus));
    assert_eq!(
        control.background,
        theme
            .control(ControlRole::Standard, ControlState::HOVERED)
            .background
    );
}

#[test]
fn disabled_unselected_control_ignores_interaction() {
    let theme = Theme::Dark;
    let disabled = theme.control(ControlRole::Standard, ControlState::DISABLED);
    let combined = ControlState::new()
        .disabled()
        .interaction(InteractionState::PRESSED.hovered().focused());

    assert_eq!(theme.control(ControlRole::Standard, combined), disabled);
}

#[test]
fn disabled_selected_control_suppresses_hover_and_pressed() {
    let theme = Theme::Dark;
    let idle_disabled_selected = theme.control(
        ControlRole::Selectable,
        ControlState::new().selected().disabled(),
    );
    let combined = ControlState::new()
        .selected()
        .disabled()
        .interaction(InteractionState::PRESSED.hovered());

    assert_eq!(
        theme.control(ControlRole::Selectable, combined),
        idle_disabled_selected
    );
}

#[test]
fn disabled_selected_control_uses_a_canonical_dimmed_selected_fill() {
    let theme = Theme::Dark;
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
    let disabled_selected = theme.control(
        ControlRole::Selectable,
        ControlState::new().selected().disabled(),
    );

    assert_eq!(
        disabled_selected.background,
        selected.background.scale_alpha(0.60)
    );
    assert_eq!(
        disabled_selected.foreground,
        selected.foreground.scale_alpha(0.60)
    );
    assert_ne!(
        disabled_selected,
        theme.control(ControlRole::Standard, ControlState::DISABLED)
    );
}

#[test]
fn selected_hover_and_pressed_intensify_the_selected_fill_centrally() {
    let theme = Theme::Dark;
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
    let selected_hovered = theme.control(
        ControlRole::Selectable,
        ControlState::new()
            .selected()
            .interaction(InteractionState::HOVERED),
    );
    let selected_pressed = theme.control(
        ControlRole::Selectable,
        ControlState::new()
            .selected()
            .interaction(InteractionState::PRESSED),
    );

    assert_eq!(
        selected_hovered.background,
        selected.background.scale_alpha(1.20)
    );
    assert_eq!(
        selected_pressed.background,
        selected.background.scale_alpha(0.88)
    );
    assert_eq!(selected_hovered.foreground, selected.foreground);
    assert_eq!(selected_pressed.foreground, selected.foreground);
}

#[test]
fn selected_focus_and_selected_idle_are_visually_distinguishable() {
    let theme = Theme::Dark;
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
    let selected_focused = theme.control(
        ControlRole::Selectable,
        ControlState::new()
            .selected()
            .interaction(InteractionState::FOCUSED),
    );

    assert_ne!(selected.border, selected_focused.border);
    assert_eq!(selected_focused.background, selected.background);
}

#[test]
fn embedded_role_leaves_its_host_surface_bare_when_untouched_or_disabled() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);

        for state in [ControlState::ENABLED, ControlState::DISABLED] {
            let fill = theme.control(ControlRole::Embedded, state).background;

            assert_eq!(
                fill.a, 0.0,
                "{mode:?} embedded {state:?} must add no emphasis beyond its host surface"
            );
        }
    }
}

#[test]
fn body_owning_roles_still_fill_themselves_when_untouched() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);

        for role in [ControlRole::Standard, ControlRole::Selectable] {
            let fill = theme.control(role, ControlState::ENABLED).background;

            assert_eq!(
                fill.a, 1.0,
                "{mode:?} {role:?} owns its body and must keep filling it at rest"
            );
        }
    }
}

#[test]
fn embedded_transient_fills_are_translucent_and_ordered() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let hover = theme
            .control(ControlRole::Embedded, ControlState::HOVERED)
            .background;
        let pressed = theme
            .control(ControlRole::Embedded, ControlState::PRESSED)
            .background;

        for (label, layer) in [("hover", hover), ("pressed", pressed)] {
            assert!(
                layer.a > 0.0 && layer.a < 1.0,
                "{mode:?} embedded {label} must be translucent to composite over any host, got {layer:?}"
            );
        }
        assert!(
            pressed.a > hover.a,
            "{mode:?} pressed must intensify past hover, got {} vs {}",
            pressed.a,
            hover.a
        );
    }
}

#[test]
fn embedded_emphasis_reads_the_same_on_every_host_surface() {
    // The point of a translucent layer: one emphasis weight everywhere, instead
    // of the opaque tokens' accidental weight, which grew stronger the darker
    // the host happened to be — 0.045 on Panel against 0.093 on Chrome.
    //
    // Calibrated against separation from the host rather than against the
    // opaque token. Matching that token on Panel is what left the layer at
    // ~0.043 on the darker Sidebar, where it read as nothing at all.
    const MIN_SEPARATION: f32 = 0.05;
    const MAX_SPREAD: f32 = 0.03;

    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let foreground = luminance(theme.text(TextRole::Primary).color);
        let mut separations = Vec::new();

        for role in [
            SurfaceRole::Panel,
            SurfaceRole::Sidebar,
            SurfaceRole::Chrome,
            SurfaceRole::Popover,
        ] {
            let host = theme.surface(role).background;
            let hovered = composite(
                theme
                    .control(ControlRole::Embedded, ControlState::HOVERED)
                    .background,
                host,
            );
            let pressed = composite(
                theme
                    .control(ControlRole::Embedded, ControlState::PRESSED)
                    .background,
                host,
            );
            let host = luminance(host);
            let separation = (luminance(hovered) - host).abs();

            assert!(
                separation > MIN_SEPARATION,
                "{mode:?} hover on {role:?} separates by only {separation:.4}"
            );
            assert_eq!(
                luminance(hovered) > host,
                foreground > host,
                "{mode:?} hover on {role:?} must move toward the foreground, not away"
            );
            assert!(
                (luminance(pressed) - host).abs() > separation,
                "{mode:?} pressed on {role:?} must intensify past hover"
            );

            separations.push(separation);
        }

        let spread = separations.iter().fold(f32::MIN, |a, b| a.max(*b))
            - separations.iter().fold(f32::MAX, |a, b| a.min(*b));
        assert!(
            spread < MAX_SPREAD,
            "{mode:?} emphasis varies by {spread:.4} across host surfaces, which is the \
             surface-dependence the layer exists to remove"
        );
    }
}

/// Idle → hover → pressed must climb monotonically *toward the foreground*.
///
/// Distance from idle is not enough: a hover that moves the wrong way clears a
/// distance check while reading as a dent rather than a lift. In dark mode the
/// body-owning roles drew hover from `surface_elevated` and pressed from
/// `mix(app, foreground, _)` — two different scales — which put pressed below
/// hover on Button, Tabs, TreeItem, SideRail, and the choice anchors.
#[test]
fn hover_then_pressed_climb_toward_the_foreground_for_every_role() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let host = theme.surface(SurfaceRole::Panel).background;
        let idle = luminance(host);
        let toward_foreground = luminance(theme.text(TextRole::Primary).color) > idle;

        for role in [
            ControlRole::Standard,
            ControlRole::Selectable,
            ControlRole::Embedded,
        ] {
            let hovered = luminance(composite(
                theme.control(role, ControlState::HOVERED).background,
                host,
            ));
            let pressed = luminance(composite(
                theme.control(role, ControlState::PRESSED).background,
                host,
            ));

            let climbs = |from: f32, to: f32| {
                if toward_foreground {
                    to > from
                } else {
                    to < from
                }
            };

            assert!(
                climbs(idle, hovered),
                "{mode:?} {role:?}: hover ({hovered:.4}) does not lift off idle ({idle:.4})"
            );
            assert!(
                climbs(hovered, pressed),
                "{mode:?} {role:?}: pressed ({pressed:.4}) does not intensify past \
                 hover ({hovered:.4})"
            );
        }
    }
}

#[test]
fn embedded_and_body_owning_roles_share_one_selected_ladder() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);

        for interaction in [
            InteractionState::NONE,
            InteractionState::HOVERED,
            InteractionState::PRESSED,
            InteractionState::FOCUSED,
        ] {
            for enabled in [true, false] {
                let mut state = ControlState::new().selected().interaction(interaction);
                if !enabled {
                    state = state.disabled();
                }

                assert_eq!(
                    theme.control(ControlRole::Embedded, state),
                    theme.control(ControlRole::Selectable, state),
                    "{mode:?} selection is role-independent, but {state:?} diverged"
                );
            }
        }
    }
}

use iced::{Color, Shadow};

use crate::tokens::shadow;

use super::super::{
    BorderRole, ControlRole, ControlState, SurfaceRole, TextRole, ThemeMode, ToneRole,
};

/// Combined-selected-state alpha ladder for [`ColorScheme::selected_control_spec`].
/// Centralized here so no widget scales these fills locally.
const SELECTED_HOVER_ALPHA: f32 = 1.20;
const SELECTED_PRESSED_ALPHA: f32 = 0.88;
const DISABLED_SELECTED_ALPHA: f32 = 0.60;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderSpec {
    pub color: Color,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceSpec {
    pub background: Color,
    pub foreground: Color,
    pub border: BorderSpec,
    pub shadow: Shadow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextSpec {
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlSpec {
    pub background: Color,
    pub foreground: Color,
    pub border: BorderSpec,
    pub focus: BorderSpec,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToneSpec {
    pub color: Color,
    pub on_color: Color,
    pub container: Color,
    pub on_container: Color,
    pub border: BorderSpec,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorScheme {
    is_dark: bool,
    surface: SurfaceColors,
    text: TextColors,
    border: BorderColors,
    control: ControlColors,
    tone: ToneColors,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SurfaceColors {
    app: Color,
    chrome: Color,
    sidebar: Color,
    panel: Color,
    elevated: Color,
    overlay: Color,
    canvas: Color,
    scrim: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextColors {
    primary: Color,
    secondary: Color,
    muted: Color,
    disabled: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BorderColors {
    subtle: Color,
    default: Color,
    strong: Color,
    focus: Color,
    accent: Color,
    danger: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ControlColors {
    active: Color,
    hover: Color,
    pressed: Color,
    disabled: Color,
    selected: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ToneColors {
    neutral: ToneColorsForRole,
    primary: ToneColorsForRole,
    info: ToneColorsForRole,
    success: ToneColorsForRole,
    warning: ToneColorsForRole,
    danger: ToneColorsForRole,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ToneColorsForRole {
    color: Color,
    on_color: Color,
    container: Color,
    on_container: Color,
    border: Color,
}

impl ColorScheme {
    pub(crate) fn from_mode(mode: ThemeMode) -> Self {
        Self::from_palette(
            super::super::palette::palette(mode),
            matches!(mode, ThemeMode::Dark),
        )
    }

    pub(crate) fn from_palette(palette: iced::theme::Palette, is_dark: bool) -> Self {
        let app = palette.background;
        let foreground = palette.text;
        let primary = palette.primary;
        let success = palette.success;
        let danger = palette.danger;

        let warning = palette.warning;

        let info = if is_dark {
            rgb(0x60A5FA)
        } else {
            rgb(0x2563EB)
        };

        // Dark ramp widened to a perceptually-separated spacing
        // (App/Chrome/Sidebar/Canvas/Panel/Elevated/Popover), guarded by the
        // ΔL* adjacency invariant test below. The top end (Panel/Popover) is
        // capped by the Accent-tone and Secondary-text contrast floors
        // asserted elsewhere in this module's test suite.
        let (chrome, sidebar, surface, surface_elevated, surface_overlay, canvas) = if is_dark {
            (
                mix(app, foreground, 0.025),
                mix(app, foreground, 0.05),
                mix(app, foreground, 0.09),
                mix(app, foreground, 0.15),
                mix(app, foreground, 0.21),
                mix(app, foreground, 0.07),
            )
        } else {
            (
                mix(app, foreground, 0.05),
                mix(app, foreground, 0.035),
                Color::WHITE,
                mix(app, foreground, 0.025),
                Color::WHITE,
                mix(app, foreground, 0.02),
            )
        };

        let text_secondary = mix(foreground, app, if is_dark { 0.22 } else { 0.32 });
        let text_muted = mix(foreground, app, if is_dark { 0.42 } else { 0.46 });
        let border_subtle = mix(app, foreground, if is_dark { 0.08 } else { 0.12 });
        let border = mix(app, foreground, if is_dark { 0.14 } else { 0.18 });
        let border_strong = mix(app, foreground, if is_dark { 0.22 } else { 0.28 });
        let primary_tone = if is_dark {
            mix(primary, Color::WHITE, 0.38)
        } else {
            primary
        };
        let focus = focus_color(primary_tone, is_dark);
        let control_hover = if is_dark {
            surface_elevated
        } else {
            mix(app, foreground, 0.06)
        };
        let control_pressed = mix(app, foreground, 0.10);
        let control_disabled = if is_dark {
            surface
        } else {
            mix(app, foreground, 0.05)
        };
        let neutral = mix(foreground, app, if is_dark { 0.38 } else { 0.42 });
        let neutral_bg = with_alpha(text_muted, if is_dark { 0.16 } else { 0.10 });
        let primary_subtle = with_alpha(primary, if is_dark { 0.16 } else { 0.12 });

        Self {
            is_dark,
            surface: SurfaceColors {
                app,
                chrome,
                sidebar,
                panel: surface,
                elevated: surface_elevated,
                overlay: surface_overlay,
                canvas,
                scrim: with_alpha(Color::BLACK, 0.45),
            },
            text: TextColors {
                primary: foreground,
                secondary: text_secondary,
                muted: text_muted,
                disabled: with_alpha(text_muted, 0.65),
            },
            border: BorderColors {
                subtle: border_subtle,
                default: border,
                strong: border_strong,
                focus,
                accent: with_alpha(primary_tone, 0.40),
                danger,
            },
            control: ControlColors {
                active: surface,
                hover: control_hover,
                pressed: control_pressed,
                disabled: control_disabled,
                selected: primary_subtle,
            },
            tone: ToneColors {
                neutral: ToneColorsForRole::new(neutral, neutral_bg, border_subtle, is_dark),
                primary: ToneColorsForRole::new(
                    primary_tone,
                    primary_subtle,
                    with_alpha(primary_tone, 0.40),
                    is_dark,
                ),
                info: ToneColorsForRole::new(
                    info,
                    tone_background(info, is_dark),
                    with_alpha(info, 0.40),
                    is_dark,
                ),
                success: ToneColorsForRole::new(
                    success,
                    tone_background(success, is_dark),
                    with_alpha(success, 0.40),
                    is_dark,
                ),
                warning: ToneColorsForRole::new(
                    warning,
                    tone_background(warning, is_dark),
                    with_alpha(warning, 0.40),
                    is_dark,
                ),
                danger: ToneColorsForRole::new(
                    danger,
                    tone_background(danger, is_dark),
                    with_alpha(danger, 0.40),
                    is_dark,
                ),
            },
        }
    }

    pub const fn is_dark(&self) -> bool {
        self.is_dark
    }

    /// Resolves a surface's fill and shadow. Borders are a separate,
    /// explicit concern (see [`BorderRole`] and each widget's own opt-in
    /// border builder) — no [`SurfaceRole`] auto-emits one here.
    pub fn surface(&self, role: SurfaceRole) -> SurfaceSpec {
        match role {
            SurfaceRole::App => {
                self.surface_spec(self.surface.app, BorderSpec::none(), shadow::NONE)
            }
            SurfaceRole::Chrome => {
                self.surface_spec(self.surface.chrome, BorderSpec::none(), shadow::NONE)
            }
            SurfaceRole::Sidebar => {
                self.surface_spec(self.surface.sidebar, BorderSpec::none(), shadow::NONE)
            }
            SurfaceRole::Panel => {
                self.surface_spec(self.surface.panel, BorderSpec::none(), shadow::NONE)
            }
            SurfaceRole::Canvas => {
                self.surface_spec(self.surface.canvas, BorderSpec::none(), shadow::NONE)
            }
            SurfaceRole::Elevated => {
                self.surface_spec(self.surface.elevated, BorderSpec::none(), shadow::ELEVATED)
            }
            SurfaceRole::Popover => {
                self.surface_spec(self.surface.overlay, BorderSpec::none(), shadow::POPOVER)
            }
            // Dialog is its own top-of-stack modal surface: the topmost
            // (Popover-tier) fill, paired with the strongest shadow in the
            // elevation ramp, rather than sharing the Panel surface.
            SurfaceRole::Dialog => {
                self.surface_spec(self.surface.overlay, BorderSpec::none(), shadow::DIALOG)
            }
            SurfaceRole::Scrim => {
                self.surface_spec(self.surface.scrim, BorderSpec::none(), shadow::NONE)
            }
        }
    }

    pub fn text(&self, role: TextRole) -> TextSpec {
        let color = match role {
            TextRole::Primary => self.text.primary,
            TextRole::Secondary => self.text.secondary,
            TextRole::Muted => self.text.muted,
            TextRole::Disabled => self.text.disabled,
        };

        TextSpec { color }
    }

    pub fn border(&self, role: BorderRole) -> BorderSpec {
        match role {
            BorderRole::Default => BorderSpec::new(self.border.default, 1.0),
            BorderRole::Subtle => BorderSpec::new(self.border.subtle, 1.0),
            BorderRole::Strong => BorderSpec::new(self.border.strong, 1.0),
            BorderRole::Accent => BorderSpec::new(self.border.accent, 1.0),
            BorderRole::Focus => BorderSpec::new(self.border.focus, 1.0),
            BorderRole::Danger => BorderSpec::new(self.border.danger, 1.0),
        }
    }

    /// Resolves the complete combined `selected × hover/pressed/focus ×
    /// disabled` control state in one place. Widgets consume this instead of
    /// scaling semantic colors locally: selection stays persistently
    /// distinguishable, hover/pressed intensify it, disabled is resolved
    /// once (suppressing hover/pressed and dimming selection by a single
    /// canonical amount, never a widget-local alpha), and focus-visible is a
    /// layout-neutral border swap independent of selection. Category-owned
    /// visual projection — which of these fields a widget actually paints —
    /// remains with the caller.
    pub fn control(&self, role: ControlRole, state: ControlState) -> ControlSpec {
        if state.selected {
            return self.selected_control_spec(state);
        }

        if !state.enabled {
            return self.control_spec(
                self.control.disabled,
                self.text.disabled,
                self.border(BorderRole::Subtle),
            );
        }

        let background = if state.interaction.pressed || state.interaction.dragged {
            self.control.pressed
        } else if state.interaction.hovered || state.interaction.focused {
            self.control.hover
        } else {
            match role {
                ControlRole::Standard | ControlRole::Selectable => self.control.active,
                ControlRole::Embedded => Color::TRANSPARENT,
            }
        };
        let border = if state.interaction.focused {
            self.border(BorderRole::Focus)
        } else if state.interaction.pressed || state.interaction.dragged {
            self.border(BorderRole::Strong)
        } else {
            self.border(BorderRole::Default)
        };

        self.control_spec(background, self.text.primary, border)
    }

    fn selected_control_spec(&self, state: ControlState) -> ControlSpec {
        let border = if state.interaction.focused {
            self.border(BorderRole::Focus)
        } else {
            self.border(BorderRole::Accent)
        };

        if !state.enabled {
            // Disabled + selected is resolved once, here: a single canonical
            // dimming of the selected fill, never a widget-local alpha.
            return self.control_spec(
                self.control.selected.scale_alpha(DISABLED_SELECTED_ALPHA),
                self.tone.primary.color.scale_alpha(DISABLED_SELECTED_ALPHA),
                border,
            );
        }

        // Disabled is checked first, so hover/pressed feedback never reaches
        // a disabled control.
        let background = if state.interaction.pressed || state.interaction.dragged {
            self.control.selected.scale_alpha(SELECTED_PRESSED_ALPHA)
        } else if state.interaction.hovered {
            self.control.selected.scale_alpha(SELECTED_HOVER_ALPHA)
        } else {
            self.control.selected
        };

        self.control_spec(background, self.tone.primary.color, border)
    }

    pub fn tone(&self, tone: ToneRole) -> ToneSpec {
        self.tone_colors(tone).spec()
    }

    fn surface_spec(&self, background: Color, border: BorderSpec, shadow: Shadow) -> SurfaceSpec {
        SurfaceSpec {
            background,
            foreground: self.text.primary,
            border,
            shadow,
        }
    }

    fn control_spec(
        &self,
        background: Color,
        foreground: Color,
        border: BorderSpec,
    ) -> ControlSpec {
        ControlSpec {
            background,
            foreground,
            border,
            focus: self.border(BorderRole::Focus),
        }
    }

    fn tone_colors(&self, tone: ToneRole) -> ToneColorsForRole {
        match tone {
            ToneRole::Neutral => self.tone.neutral,
            ToneRole::Accent => self.tone.primary,
            ToneRole::Info => self.tone.info,
            ToneRole::Success => self.tone.success,
            ToneRole::Warning => self.tone.warning,
            ToneRole::Danger => self.tone.danger,
        }
    }
}

impl ToneColorsForRole {
    fn new(color: Color, container: Color, border: Color, is_dark: bool) -> Self {
        Self {
            color,
            on_color: readable_on(color, is_dark),
            container,
            on_container: readable_on_container(color, is_dark),
            border,
        }
    }

    fn spec(self) -> ToneSpec {
        ToneSpec {
            color: self.color,
            on_color: self.on_color,
            container: self.container,
            on_container: self.on_container,
            border: BorderSpec::new(self.border, 1.0),
        }
    }
}

impl BorderSpec {
    pub const fn new(color: Color, width: f32) -> Self {
        Self { color, width }
    }

    pub const fn none() -> Self {
        Self {
            color: Color::TRANSPARENT,
            width: 0.0,
        }
    }
}

fn rgb(hex: u32) -> Color {
    Color::from_rgb(
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    )
}

fn mix(from: Color, to: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        r: from.r + (to.r - from.r) * amount,
        g: from.g + (to.g - from.g) * amount,
        b: from.b + (to.b - from.b) * amount,
        a: from.a + (to.a - from.a) * amount,
    }
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color {
        a: alpha.clamp(0.0, 1.0),
        ..color
    }
}

fn focus_color(primary: Color, is_dark: bool) -> Color {
    let brightness = if is_dark { 0.16 } else { 0.08 };

    with_alpha(mix(primary, Color::WHITE, brightness), 0.92)
}

fn tone_background(color: Color, is_dark: bool) -> Color {
    with_alpha(color, if is_dark { 0.16 } else { 0.10 })
}

fn readable_on(background: Color, _is_dark: bool) -> Color {
    if crate::theme::color::contrast_ratio(Color::WHITE, background)
        >= crate::theme::color::contrast_ratio(Color::BLACK, background)
    {
        Color::WHITE
    } else {
        Color::BLACK
    }
}

fn readable_on_container(color: Color, _is_dark: bool) -> Color {
    color
}

#[cfg(test)]
fn luminance(color: Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}

#[cfg(test)]
mod color_scheme_tests {
    use super::*;
    use crate::theme::{InteractionState, Theme, ThemeMode};

    const REPRESENTATIVE_SURFACES: [SurfaceRole; 8] = [
        SurfaceRole::App,
        SurfaceRole::Chrome,
        SurfaceRole::Sidebar,
        SurfaceRole::Panel,
        SurfaceRole::Elevated,
        SurfaceRole::Canvas,
        SurfaceRole::Dialog,
        SurfaceRole::Popover,
    ];
    const MUTED_FLOOR_SURFACES: [SurfaceRole; 5] = [
        SurfaceRole::App,
        SurfaceRole::Chrome,
        SurfaceRole::Sidebar,
        SurfaceRole::Canvas,
        SurfaceRole::Panel,
    ];
    const MUTED_CONTRAST_FLOOR: f32 = 3.0;
    const TEXT_ROLE_CASES: [(TextRole, f32); 4] = [
        (TextRole::Primary, 4.5),
        (TextRole::Secondary, 4.5),
        (TextRole::Muted, 2.25),
        (TextRole::Disabled, 2.0),
    ];
    const TONE_ROLES: [ToneRole; 6] = [
        ToneRole::Neutral,
        ToneRole::Accent,
        ToneRole::Info,
        ToneRole::Success,
        ToneRole::Warning,
        ToneRole::Danger,
    ];
    const MIN_TONE_CONTRAST: f32 = 2.25;
    const MIN_ON_ACCENT_CONTRAST: f32 = 4.5;

    #[test]
    fn light_and_dark_modes_are_explicit() {
        assert!(!Theme::Light.color_scheme().is_dark());
        assert!(Theme::Dark.color_scheme().is_dark());
    }

    #[test]
    fn surfaces_use_distinct_shell_chrome() {
        let theme = Theme::Dark;

        assert_ne!(
            theme.surface(SurfaceRole::Chrome).background,
            theme.surface(SurfaceRole::Panel).background
        );
        assert_ne!(
            theme.surface(SurfaceRole::Sidebar).background,
            theme.surface(SurfaceRole::Chrome).background
        );
    }

    const ORDERED_STRUCTURAL_SURFACES: [SurfaceRole; 7] = [
        SurfaceRole::App,
        SurfaceRole::Chrome,
        SurfaceRole::Sidebar,
        SurfaceRole::Canvas,
        SurfaceRole::Panel,
        SurfaceRole::Elevated,
        SurfaceRole::Popover,
    ];
    const MIN_ADJACENT_DELTA_L: f32 = 1.5;

    fn shadow_prominence(shadow: Shadow) -> f32 {
        shadow.color.a * shadow.blur_radius
    }

    #[test]
    fn dialog_no_longer_shares_the_panel_fill() {
        let theme = Theme::Dark;

        assert_ne!(
            theme.surface(SurfaceRole::Dialog).background,
            theme.surface(SurfaceRole::Panel).background
        );
    }

    #[test]
    fn elevation_shadow_ramp_increases_toward_dialog() {
        let theme = Theme::Dark;

        let elevated = shadow_prominence(theme.surface(SurfaceRole::Elevated).shadow);
        let popover = shadow_prominence(theme.surface(SurfaceRole::Popover).shadow);
        let dialog = shadow_prominence(theme.surface(SurfaceRole::Dialog).shadow);

        assert!(
            elevated < popover,
            "Elevated should be less prominent than Popover"
        );
        assert!(
            popover < dialog,
            "Popover should be less prominent than Dialog"
        );
    }

    #[test]
    fn dark_surface_ramp_preserves_semantic_lightness_order() {
        let theme = Theme::Dark;
        let lightness = |role: SurfaceRole| {
            crate::theme::color::perceptual_lightness(theme.surface(role).background)
        };

        for pair in ORDERED_STRUCTURAL_SURFACES.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            assert!(
                lightness(a) < lightness(b),
                "expected {a:?} lightness to be less than {b:?}"
            );
        }
    }

    #[test]
    fn dark_surface_ramp_clears_the_adjacency_floor() {
        let theme = Theme::Dark;
        let lightness = |role: SurfaceRole| {
            crate::theme::color::perceptual_lightness(theme.surface(role).background)
        };

        for pair in ORDERED_STRUCTURAL_SURFACES.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let delta = lightness(b) - lightness(a);

            assert!(
                delta >= MIN_ADJACENT_DELTA_L,
                "{a:?} -> {b:?} ΔL* {delta:.2} is below the {MIN_ADJACENT_DELTA_L:.2} floor"
            );
        }
    }

    #[test]
    fn tones_are_visual_not_domain_specific() {
        let theme = Theme::Dark;

        assert_eq!(
            theme.tone(ToneRole::Accent).color,
            theme
                .control(ControlRole::Selectable, ControlState::SELECTED)
                .foreground
        );
        assert_eq!(
            theme.tone(ToneRole::Accent).border,
            theme
                .control(ControlRole::Selectable, ControlState::SELECTED)
                .border
        );
    }

    #[test]
    fn accent_tone_reuses_the_palette_primary_pair() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let scheme = Theme::from_mode(mode).color_scheme();

            assert_eq!(scheme.tone(ToneRole::Accent), scheme.tone.primary.spec());
        }
    }

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
    fn focus_border_is_brighter_than_accent() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::from_mode(mode);
            let focus = theme.border(BorderRole::Focus).color;
            let accent = theme.tone(ToneRole::Accent).color;

            assert!(luminance(focus) > luminance(accent));
            assert!(focus.a > 0.90);
        }
    }

    #[test]
    fn focus_border_stays_visually_close_to_accent() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::from_mode(mode);
            let focus = theme.border(BorderRole::Focus).color;
            let accent = theme.tone(ToneRole::Accent).color;

            assert!(color_distance(focus, accent) < 0.22);
        }
    }

    #[test]
    fn text_roles_keep_readable_contrast_on_representative_surfaces() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::from_mode(mode);

            for surface_role in REPRESENTATIVE_SURFACES {
                let surface = theme.surface(surface_role);

                for (text_role, minimum) in TEXT_ROLE_CASES {
                    let text = theme.text(text_role);
                    assert_contrast_at_least(
                        format!("{mode:?} {text_role:?} over {surface_role:?}"),
                        text.color,
                        surface.background,
                        minimum,
                    );
                }
            }
        }
    }

    #[test]
    fn muted_text_clears_the_3_to_1_floor_on_base_surfaces() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::from_mode(mode);
            let muted = theme.text(TextRole::Muted).color;

            for surface_role in MUTED_FLOOR_SURFACES {
                let background = theme.surface(surface_role).background;
                assert_contrast_at_least(
                    format!("{mode:?} Muted over {surface_role:?}"),
                    muted,
                    background,
                    MUTED_CONTRAST_FLOOR,
                );
            }
        }
    }

    #[test]
    fn muted_text_stays_below_secondary_contrast() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::from_mode(mode);
            let muted = theme.text(TextRole::Muted).color;
            let secondary = theme.text(TextRole::Secondary).color;

            for surface_role in MUTED_FLOOR_SURFACES {
                let background = theme.surface(surface_role).background;
                let muted_contrast = crate::theme::color::contrast_ratio(muted, background);
                let secondary_contrast = crate::theme::color::contrast_ratio(secondary, background);

                assert!(
                    muted_contrast < secondary_contrast,
                    "{mode:?} Muted over {surface_role:?} contrast {muted_contrast:.2} \
                     should stay below Secondary contrast {secondary_contrast:.2}"
                );
            }
        }
    }

    #[test]
    fn tone_roles_keep_readable_foreground_background_contrast() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::from_mode(mode);
            let panel_background = theme.surface(SurfaceRole::Panel).background;

            for tone_role in TONE_ROLES {
                let tone = theme.tone(tone_role);
                let background = composite_over(tone.container, panel_background);

                assert_contrast_at_least(
                    format!("{mode:?} {tone_role:?} tone"),
                    tone.color,
                    background,
                    MIN_TONE_CONTRAST,
                );
            }
        }
    }

    #[test]
    fn on_accent_text_reads_over_accent_in_light_and_dark_modes() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::from_mode(mode);
            let accent = theme.tone(ToneRole::Accent).color;

            assert_contrast_at_least(
                format!("{mode:?} on Accent"),
                theme.tone(ToneRole::Accent).on_color,
                accent,
                MIN_ON_ACCENT_CONTRAST,
            );
        }
    }

    fn assert_contrast_at_least(label: String, foreground: Color, background: Color, minimum: f32) {
        let contrast = crate::theme::color::contrast_ratio(foreground, background);

        assert!(
            contrast >= minimum,
            "{label} contrast {contrast:.2} is below {minimum:.2}"
        );
    }

    fn color_distance(first: Color, second: Color) -> f32 {
        let dr = first.r - second.r;
        let dg = first.g - second.g;
        let db = first.b - second.b;

        (dr * dr + dg * dg + db * db).sqrt()
    }

    fn composite_over(foreground: Color, background: Color) -> Color {
        Color {
            r: foreground.r * foreground.a + background.r * (1.0 - foreground.a),
            g: foreground.g * foreground.a + background.g * (1.0 - foreground.a),
            b: foreground.b * foreground.a + background.b * (1.0 - foreground.a),
            a: foreground.a + background.a * (1.0 - foreground.a),
        }
    }
}

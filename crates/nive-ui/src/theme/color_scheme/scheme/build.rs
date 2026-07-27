use iced::{Color, Shadow};

use super::helpers::{
    focus_color, mix, readable_on, readable_on_container, rgb, tone_background, with_alpha,
};
use super::{
    BorderColors, BorderSpec, ColorScheme, ControlColors, ControlSpec, SurfaceColors, SurfaceSpec,
    TextColors, TextSpec, ToneColors, ToneColorsForRole, ToneSpec, DISABLED_SELECTED_ALPHA,
    SELECTED_HOVER_ALPHA, SELECTED_PRESSED_ALPHA,
};
use crate::theme::{
    BorderRole, ControlRole, ControlState, SurfaceRole, TextRole, ThemeMode, ToneRole,
};
use crate::tokens::shadow;

/// The neutral (unselected) fills one control role paints per interaction
/// state, resolved once so [`ColorScheme::control`] keeps a single state
/// precedence for every role.
struct RoleFills {
    idle: Color,
    hover: Color,
    pressed: Color,
    disabled: Color,
}

impl ColorScheme {
    pub(crate) fn from_mode(mode: ThemeMode) -> Self {
        Self::from_palette(
            crate::theme::palette::palette(mode),
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
        // Translucent so they composite over whichever surface hosts the
        // control instead of over `app`, giving one emphasis weight on every
        // surface rather than the opaque tokens' accidental one, which grew
        // stronger the darker the host happened to be.
        //
        // Calibrated on separation from the host, not on matching the opaque
        // token: matching it on Panel left only ~0.043 luminance of separation,
        // which reads as nothing on the darker Sidebar where trees and rails
        // live. These land near 0.07 on every surface, matching what light mode
        // already achieved. Light needs more alpha for the same perceived
        // weight, because it darkens a near-white host rather than lightening a
        // near-black one.
        let control_hover_layer = with_alpha(foreground, if is_dark { 0.10 } else { 0.09 });
        let control_pressed_layer = with_alpha(foreground, if is_dark { 0.15 } else { 0.13 });
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
                hover_layer: control_hover_layer,
                pressed_layer: control_pressed_layer,
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

    /// Whether the role owns chrome of its own, or paints on top of whatever
    /// surface hosts it. `Embedded` does the latter, so its untouched states
    /// contribute nothing and its transient states are translucent layers that
    /// composite correctly over every host from Panel to Popover.
    fn role_fills(&self, role: ControlRole) -> RoleFills {
        match role {
            ControlRole::Standard | ControlRole::Selectable => RoleFills {
                idle: self.control.active,
                hover: self.control.hover,
                pressed: self.control.pressed,
                disabled: self.control.disabled,
            },
            ControlRole::Embedded => RoleFills {
                idle: Color::TRANSPARENT,
                hover: self.control.hover_layer,
                pressed: self.control.pressed_layer,
                disabled: Color::TRANSPARENT,
            },
        }
    }

    /// Resolves the complete combined `selected × hover/pressed/focus ×
    /// disabled` control state in one place. Widgets consume this instead of
    /// scaling semantic colors locally: selection stays persistently
    /// distinguishable, hover/pressed intensify it, disabled is resolved
    /// once (suppressing hover/pressed and dimming selection by a single
    /// canonical amount, never a widget-local alpha), and focus-visible is a
    /// layout-neutral border swap independent of selection. `role` chooses
    /// which neutral fills that precedence draws from, never the precedence
    /// itself. Category-owned visual projection — which of these fields a
    /// widget actually paints — remains with the caller.
    pub fn control(&self, role: ControlRole, state: ControlState) -> ControlSpec {
        if state.selected {
            return self.selected_control_spec(state);
        }

        let fills = self.role_fills(role);

        if !state.enabled {
            return self.control_spec(
                fills.disabled,
                self.text.disabled,
                self.border(BorderRole::Subtle),
            );
        }

        let background = if state.interaction.pressed || state.interaction.dragged {
            fills.pressed
        } else if state.interaction.hovered || state.interaction.focused {
            fills.hover
        } else {
            fills.idle
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

    pub(super) fn selected_control_spec(&self, state: ControlState) -> ControlSpec {
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

    pub(super) fn surface_spec(
        &self,
        background: Color,
        border: BorderSpec,
        shadow: Shadow,
    ) -> SurfaceSpec {
        SurfaceSpec {
            background,
            foreground: self.text.primary,
            border,
            shadow,
        }
    }

    pub(super) fn control_spec(
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

    pub(super) fn tone_colors(&self, tone: ToneRole) -> ToneColorsForRole {
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
    pub(super) fn new(color: Color, container: Color, border: Color, is_dark: bool) -> Self {
        Self {
            color,
            on_color: readable_on(color, is_dark),
            container,
            on_container: readable_on_container(color, is_dark),
            border,
        }
    }

    pub(super) fn spec(self) -> ToneSpec {
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

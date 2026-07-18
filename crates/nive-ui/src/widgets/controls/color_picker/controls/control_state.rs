use std::cell::Cell;

use iced::{advanced::widget::tree, widget::canvas, Color};

use crate::advanced::focus::FocusState;

pub(super) struct ControlState {
    dragging: bool,
    focus: FocusState,
    surface_cache: canvas::Cache<iced::Renderer>,
    surface_key: Cell<Option<SurfaceCacheKey>>,
}

impl ControlState {
    pub(super) fn tag() -> tree::Tag {
        tree::Tag::of::<Self>()
    }

    pub(super) fn new_state() -> tree::State {
        tree::State::new(Self::default())
    }

    pub(super) fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub(super) fn set_dragging(&mut self, dragging: bool) {
        self.dragging = dragging;
    }

    pub(super) fn is_active(&self) -> bool {
        self.focus.is_active()
    }

    pub(super) fn is_focus_visible(&self) -> bool {
        self.focus.is_focus_visible()
    }

    pub(super) fn focus(&mut self) -> &mut FocusState {
        &mut self.focus
    }

    pub(super) fn surface_cache(&self, key: SurfaceCacheKey) -> &canvas::Cache<iced::Renderer> {
        if self.surface_key.get() != Some(key) {
            self.surface_cache.clear();
            self.surface_key.set(Some(key));
        }

        &self.surface_cache
    }
}

impl Default for ControlState {
    fn default() -> Self {
        Self {
            dragging: false,
            focus: FocusState::default(),
            surface_cache: canvas::Cache::new(),
            surface_key: Cell::new(None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SurfaceCacheKey {
    SaturationValue { hue: u32 },
    Hue,
    Alpha { red: u32, green: u32, blue: u32 },
}

impl SurfaceCacheKey {
    pub(super) fn saturation_value(hue: f32) -> Self {
        Self::SaturationValue {
            hue: finite_bits(hue.rem_euclid(360.0)),
        }
    }

    pub(super) fn alpha(color: Color) -> Self {
        Self::Alpha {
            red: finite_bits(color.r.clamp(0.0, 1.0)),
            green: finite_bits(color.g.clamp(0.0, 1.0)),
            blue: finite_bits(color.b.clamp(0.0, 1.0)),
        }
    }
}

fn finite_bits(value: f32) -> u32 {
    if value.is_finite() {
        value.to_bits()
    } else {
        0
    }
}

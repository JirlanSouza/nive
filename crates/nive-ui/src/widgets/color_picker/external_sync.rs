use std::collections::VecDeque;

use iced::Color;

const LOCAL_ECHO_BUFFER: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum ExternalSync {
    Unchanged,
    LocalEcho,
    Changed(Color),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ExternalColorSync {
    last_external: Color,
    pending_local: VecDeque<Color>,
}

impl ExternalColorSync {
    pub(super) fn new(value: Color) -> Self {
        Self {
            last_external: value,
            pending_local: VecDeque::new(),
        }
    }

    pub(super) fn sync(&mut self, value: Color) -> ExternalSync {
        if colors_match(value, self.last_external) {
            return ExternalSync::Unchanged;
        }

        if self.remove_pending_local(value) {
            self.last_external = value;
            return ExternalSync::LocalEcho;
        }

        self.last_external = value;
        self.pending_local.clear();
        ExternalSync::Changed(value)
    }

    pub(super) fn accept_local_change(&mut self, color: Color) {
        self.pending_local.push_back(color);

        while self.pending_local.len() > LOCAL_ECHO_BUFFER {
            self.pending_local.pop_front();
        }
    }

    fn remove_pending_local(&mut self, value: Color) -> bool {
        let Some(index) = self
            .pending_local
            .iter()
            .position(|pending| colors_match(value, *pending))
        else {
            return false;
        };

        self.pending_local.remove(index);
        true
    }
}

fn colors_match(left: Color, right: Color) -> bool {
    const EPSILON: f32 = 1.0 / 255.0;

    (left.r - right.r).abs() <= EPSILON
        && (left.g - right.g).abs() <= EPSILON
        && (left.b - right.b).abs() <= EPSILON
        && (left.a - right.a).abs() <= EPSILON
}

#[cfg(test)]
mod external_color_sync_tests {
    use super::*;

    #[test]
    fn unchanged_external_value_is_ignored() {
        let color = Color::from_rgb(0.2, 0.4, 0.6);
        let mut sync = ExternalColorSync::new(color);

        assert_eq!(sync.sync(color), ExternalSync::Unchanged);
    }

    #[test]
    fn pending_local_echo_is_accepted_without_external_change() {
        let initial = Color::from_rgb(0.2, 0.4, 0.6);
        let local = Color::from_rgb(0.8, 0.4, 0.6);
        let mut sync = ExternalColorSync::new(initial);

        sync.accept_local_change(local);

        assert_eq!(sync.sync(local), ExternalSync::LocalEcho);
    }

    #[test]
    fn different_external_value_is_reported_as_changed() {
        let initial = Color::from_rgb(0.2, 0.4, 0.6);
        let external = Color::from_rgb(0.8, 0.4, 0.6);
        let mut sync = ExternalColorSync::new(initial);

        assert_eq!(sync.sync(external), ExternalSync::Changed(external));
    }

    #[test]
    fn local_echo_matches_quantized_parent_value() {
        let local = Color::from_rgba(0.33, 0.33, 0.33, 0.5);
        let quantized = Color::from_rgba(84.0 / 255.0, 84.0 / 255.0, 84.0 / 255.0, 128.0 / 255.0);
        let mut sync = ExternalColorSync::new(Color::BLACK);

        sync.accept_local_change(local);

        assert_eq!(sync.sync(quantized), ExternalSync::LocalEcho);
    }
}

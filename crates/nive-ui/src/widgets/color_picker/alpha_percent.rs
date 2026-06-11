#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct AlphaPercentDraft {
    value: String,
}

impl AlphaPercentDraft {
    pub(super) fn as_str(&self) -> &str {
        self.value.as_str()
    }

    pub(super) fn sync_alpha(&mut self, alpha: f32) {
        self.value = format!("{}", (alpha.clamp(0.0, 1.0) * 100.0).round() as u8);
    }

    pub(super) fn input(&mut self, value: String) -> Option<f32> {
        self.value = sanitize_alpha_percent(&value);
        self.alpha()
    }

    pub(super) fn alpha(&self) -> Option<f32> {
        if self.value.is_empty() {
            return None;
        }

        self.value
            .parse::<u8>()
            .ok()
            .map(|value| f32::from(value) / 100.0)
    }
}

fn sanitize_alpha_percent(value: &str) -> String {
    let digits: String = value
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(3)
        .collect();

    if digits.is_empty() {
        return String::new();
    }

    digits
        .parse::<u16>()
        .map(|value| value.min(100).to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod alpha_percent_tests {
    use super::*;

    #[test]
    fn syncs_alpha_as_whole_percent() {
        let mut draft = AlphaPercentDraft::default();

        draft.sync_alpha(0.8);

        assert_eq!(draft.as_str(), "80");
    }

    #[test]
    fn input_clamps_values_above_one_hundred() {
        let mut draft = AlphaPercentDraft::default();

        let alpha = draft.input("250".into());

        assert_eq!(draft.as_str(), "100");
        assert_eq!(alpha, Some(1.0));
    }

    #[test]
    fn input_removes_non_digits() {
        let mut draft = AlphaPercentDraft::default();

        let alpha = draft.input("8o%".into());

        assert_eq!(draft.as_str(), "8");
        assert_eq!(alpha, Some(0.08));
    }

    #[test]
    fn empty_input_is_allowed_as_a_draft() {
        let mut draft = AlphaPercentDraft::default();

        let alpha = draft.input("abc".into());

        assert_eq!(draft.as_str(), "");
        assert_eq!(alpha, None);
    }
}

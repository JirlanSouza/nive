/// Semantic Dialog width. `Sm` is the default.
///
/// Resolved widths are requested targets, clamped by [`DialogHost`](super::super::DialogHost)
/// to the safe viewport width before rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DialogSize {
    #[default]
    Sm,
    Md,
    Lg,
}

impl DialogSize {
    /// Requested (unclamped) target width in pixels.
    pub(super) const fn target_width(self) -> f32 {
        match self {
            Self::Sm => 420.0,
            Self::Md => 560.0,
            Self::Lg => 720.0,
        }
    }
}

#[cfg(test)]
mod dialog_size_tests {
    use super::*;

    #[test]
    fn sizes_resolve_to_requested_widths() {
        assert_eq!(DialogSize::Sm.target_width(), 420.0);
        assert_eq!(DialogSize::Md.target_width(), 560.0);
        assert_eq!(DialogSize::Lg.target_width(), 720.0);
    }

    #[test]
    fn default_is_sm() {
        assert_eq!(DialogSize::default(), DialogSize::Sm);
    }
}

use iced::widget::scrollable::Scrollbar;

/// Native floating scrollbar geometry for Nive structural scrollables.
///
/// The lane remains 12 logical pixels for interaction while the thumb is a
/// fixed 6 pixels in every state. The native Iced catalog cannot vary thumb
/// width by state, so hover and drag are expressed through semantic color.
/// No embedded spacing is configured, so content width is not reserved.
pub fn overlay_scrollbar() -> Scrollbar {
    Scrollbar::new().width(12).scroller_width(6)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::scrollable::Direction;

    #[test]
    fn helper_builds_native_overlay_geometry() {
        let scrollbar = overlay_scrollbar();
        let direction = Direction::Vertical(scrollbar);

        assert_eq!(direction.vertical(), Some(&scrollbar));
        assert_eq!(scrollbar, Scrollbar::new().width(12).scroller_width(6));
        assert_ne!(
            scrollbar,
            Scrollbar::new().width(12).scroller_width(6).spacing(0)
        );
    }
}

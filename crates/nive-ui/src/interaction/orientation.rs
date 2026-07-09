use iced::{Point, Size};

/// Shared horizontal/vertical orientation for interactive widgets.
///
/// `Horizontal` means content is laid out side by side (left/right), matching
/// GTK's `GtkOrientable` and Qt's `QSplitter` semantics. This is the opposite
/// of iced's `pane_grid::Axis`, where `Axis::Horizontal` describes a
/// horizontal split *line* (panes stacked top/bottom) a false friend to
/// watch for when porting `pane_grid` code onto widgets that consume this
/// type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// Panes/content are laid out side by side (left/right).
    #[default]
    Horizontal,
    /// Panes/content are stacked top/bottom.
    Vertical,
}

impl Orientation {
    /// Projects a point onto this orientation's main axis.
    pub fn main_position(self, point: Point) -> f32 {
        match self {
            Self::Horizontal => point.x,
            Self::Vertical => point.y,
        }
    }

    /// Projects a point onto this orientation's cross axis.
    pub fn cross_position(self, point: Point) -> f32 {
        match self {
            Self::Horizontal => point.y,
            Self::Vertical => point.x,
        }
    }

    /// Projects a size onto this orientation's main axis.
    pub fn main_length(self, size: Size) -> f32 {
        match self {
            Self::Horizontal => size.width,
            Self::Vertical => size.height,
        }
    }

    /// Projects a size onto this orientation's cross axis.
    pub fn cross_length(self, size: Size) -> f32 {
        match self {
            Self::Horizontal => size.height,
            Self::Vertical => size.width,
        }
    }

    /// Builds a size from main-axis and cross-axis lengths.
    pub fn size(self, main: f32, cross: f32) -> Size {
        match self {
            Self::Horizontal => Size::new(main, cross),
            Self::Vertical => Size::new(cross, main),
        }
    }
}

#[cfg(test)]
mod orientation_tests {
    use super::*;

    #[test]
    fn horizontal_projects_x_and_width_as_main_axis() {
        let point = Point::new(12.0, 24.0);
        let size = Size::new(120.0, 40.0);

        assert_eq!(Orientation::Horizontal.main_position(point), 12.0);
        assert_eq!(Orientation::Horizontal.cross_position(point), 24.0);
        assert_eq!(Orientation::Horizontal.main_length(size), 120.0);
        assert_eq!(Orientation::Horizontal.cross_length(size), 40.0);
        assert_eq!(Orientation::Horizontal.size(120.0, 40.0), size);
    }

    #[test]
    fn vertical_projects_y_and_height_as_main_axis() {
        let point = Point::new(12.0, 24.0);
        let size = Size::new(120.0, 40.0);

        assert_eq!(Orientation::Vertical.main_position(point), 24.0);
        assert_eq!(Orientation::Vertical.cross_position(point), 12.0);
        assert_eq!(Orientation::Vertical.main_length(size), 40.0);
        assert_eq!(Orientation::Vertical.cross_length(size), 120.0);
        assert_eq!(Orientation::Vertical.size(40.0, 120.0), size);
    }
}

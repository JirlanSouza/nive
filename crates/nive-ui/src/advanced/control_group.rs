use iced::border::Radius;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SlotPosition {
    Single,
    First,
    Middle,
    Last,
}

pub(crate) fn position_for_index(index: usize, len: usize) -> SlotPosition {
    match (index, len) {
        (_, 1) => SlotPosition::Single,
        (0, _) => SlotPosition::First,
        (index, len) if index + 1 == len => SlotPosition::Last,
        _ => SlotPosition::Middle,
    }
}

pub(crate) fn radius_for_position(position: SlotPosition, radius: f32) -> Radius {
    match position {
        SlotPosition::Single => Radius::new(radius),
        SlotPosition::First => Radius::default().left(radius),
        SlotPosition::Middle => Radius::default(),
        SlotPosition::Last => Radius::default().right(radius),
    }
}

#[cfg(test)]
mod control_group_tests {
    use super::*;

    #[test]
    fn single_item_group_is_single_position() {
        assert_eq!(position_for_index(0, 1), SlotPosition::Single);
    }

    #[test]
    fn multi_item_group_maps_edges_and_middle() {
        assert_eq!(position_for_index(0, 3), SlotPosition::First);
        assert_eq!(position_for_index(1, 3), SlotPosition::Middle);
        assert_eq!(position_for_index(2, 3), SlotPosition::Last);
    }

    #[test]
    fn two_item_group_maps_correctly() {
        assert_eq!(position_for_index(0, 2), SlotPosition::First);
        assert_eq!(position_for_index(1, 2), SlotPosition::Last);
    }

    #[test]
    fn single_position_rounds_all_corners() {
        let r = radius_for_position(SlotPosition::Single, 6.0);
        assert_eq!(r.top_left, 6.0);
        assert_eq!(r.top_right, 6.0);
        assert_eq!(r.bottom_right, 6.0);
        assert_eq!(r.bottom_left, 6.0);
    }

    #[test]
    fn first_position_rounds_left_corners_only() {
        let r = radius_for_position(SlotPosition::First, 6.0);
        assert_eq!(r.top_left, 6.0);
        assert_eq!(r.bottom_left, 6.0);
        assert_eq!(r.top_right, 0.0);
        assert_eq!(r.bottom_right, 0.0);
    }

    #[test]
    fn last_position_rounds_right_corners_only() {
        let r = radius_for_position(SlotPosition::Last, 6.0);
        assert_eq!(r.top_left, 0.0);
        assert_eq!(r.bottom_left, 0.0);
        assert_eq!(r.top_right, 6.0);
        assert_eq!(r.bottom_right, 6.0);
    }

    #[test]
    fn middle_position_rounds_no_corners() {
        let r = radius_for_position(SlotPosition::Middle, 6.0);
        assert_eq!(r.top_left, 0.0);
        assert_eq!(r.top_right, 0.0);
        assert_eq!(r.bottom_right, 0.0);
        assert_eq!(r.bottom_left, 0.0);
    }
}

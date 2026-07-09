use iced::{keyboard, Point, Rectangle};

use super::{Orientation, TransferData, TransferOperation, TransferOperations};

/// Drag source description shared by widgets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drag<T, Origin = ()> {
    /// Payload transferred by the drag.
    pub payload: TransferData<T>,
    /// App or widget source identity.
    pub origin: Origin,
    /// Operations allowed by the source.
    pub operations: TransferOperations,
    /// Preferred source operation when no modifier requests an allowed one.
    pub preferred: TransferOperation,
}

/// Drop target probe context.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropContext<'a, T, Origin = ()> {
    /// Drag payload.
    pub payload: &'a TransferData<T>,
    /// Drag source identity.
    pub origin: &'a Origin,
    /// Operations allowed by the source.
    pub operations: TransferOperations,
    /// Source preferred operation when no modifier requests an allowed one.
    pub preferred: TransferOperation,
    /// Operation requested by current modifiers, if any.
    pub requested: Option<TransferOperation>,
    /// Current pointer position.
    pub position: Point,
    /// Current keyboard modifiers.
    pub modifiers: keyboard::Modifiers,
}

impl<T, Origin> DropContext<'_, T, Origin> {
    /// Returns the source operation preferred for this context.
    pub fn preferred_operation(&self) -> Option<TransferOperation> {
        self.operations.preferred(self.requested, self.preferred)
    }
}

/// Drop target decision carrying a widget-specific target.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DropDecision<Target> {
    /// Reject the current payload or operation.
    Reject,
    /// Accept with a widget-specific target and effective operation.
    Accept {
        /// Widget-specific target.
        target: Target,
        /// Effective operation.
        operation: TransferOperation,
    },
}

impl<Target> DropDecision<Target> {
    /// Accepts a drop target with an effective operation.
    pub fn accept(target: Target, operation: TransferOperation) -> Self {
        Self::Accept { target, operation }
    }

    /// Returns whether the decision accepts the drop.
    pub fn is_accept(&self) -> bool {
        matches!(self, Self::Accept { .. })
    }
}

/// Successful drop commit emitted by widgets.
#[derive(Debug, Clone, PartialEq)]
pub struct DropCommit<T, Origin = (), Target = ()> {
    /// Transferred payload.
    pub payload: TransferData<T>,
    /// Drag source identity.
    pub origin: Origin,
    /// Accepted widget-specific target.
    pub target: Target,
    /// Effective operation.
    pub operation: TransferOperation,
    /// Release position.
    pub position: Point,
}

/// Before/after insertion point for flat ordered collections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinearInsertion<Id> {
    /// Insert before the target item.
    Before(Id),
    /// Insert after the target item.
    After(Id),
}

/// Returns a flat before/after insertion target for a pointer position.
///
/// Exact item midpoints are treated as `After`; exact gap midpoints choose
/// `After` the previous item.
pub fn linear_insertion<Id>(
    orientation: Orientation,
    pointer: Point,
    items: impl IntoIterator<Item = (Id, Rectangle)>,
) -> Option<LinearInsertion<Id>>
where
    Id: Clone,
{
    let items: Vec<(Id, Rectangle)> = items.into_iter().collect();
    let (first_id, first_bounds) = items.first()?;
    let pointer_main = orientation.main_position(pointer);
    let first_start = orientation.main_position(first_bounds.position());

    if pointer_main < first_start {
        return Some(LinearInsertion::Before(first_id.clone()));
    }

    for window in items.windows(2) {
        let (id, bounds) = &window[0];
        let (next_id, next_bounds) = &window[1];

        if let Some(insertion) = insertion_inside_item(orientation, pointer_main, id, *bounds) {
            return Some(insertion);
        }

        let end = item_end(orientation, *bounds);
        let next_start = orientation.main_position(next_bounds.position());

        if pointer_main >= end && pointer_main < next_start {
            let gap_midpoint = end + (next_start - end) / 2.0;

            return Some(if pointer_main <= gap_midpoint {
                LinearInsertion::After(id.clone())
            } else {
                LinearInsertion::Before(next_id.clone())
            });
        }
    }

    let (last_id, last_bounds) = items.last()?;

    insertion_inside_item(orientation, pointer_main, last_id, *last_bounds)
        .or_else(|| Some(LinearInsertion::After(last_id.clone())))
}

mod session;

#[allow(unused_imports)]
pub(crate) use session::*;

fn insertion_inside_item<Id>(
    orientation: Orientation,
    pointer_main: f32,
    id: &Id,
    bounds: Rectangle,
) -> Option<LinearInsertion<Id>>
where
    Id: Clone,
{
    let start = orientation.main_position(bounds.position());
    let end = item_end(orientation, bounds);

    if pointer_main < start || pointer_main > end {
        return None;
    }

    let midpoint = start + (end - start) / 2.0;

    Some(if pointer_main < midpoint {
        LinearInsertion::Before(id.clone())
    } else {
        LinearInsertion::After(id.clone())
    })
}

fn item_end(orientation: Orientation, bounds: Rectangle) -> f32 {
    orientation.main_position(bounds.position()) + orientation.main_length(bounds.size())
}

#[cfg(test)]
mod linear_insertion_tests {
    use super::*;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle::new(Point::new(x, y), iced::Size::new(width, height))
    }

    #[test]
    fn empty_collection_has_no_target() {
        let items: Vec<(u8, Rectangle)> = Vec::new();

        assert_eq!(
            linear_insertion(Orientation::Horizontal, Point::ORIGIN, items),
            None
        );
    }

    #[test]
    fn edges_target_first_and_last_item() {
        let items = [
            (1, rect(10.0, 0.0, 20.0, 10.0)),
            (2, rect(40.0, 0.0, 20.0, 10.0)),
        ];

        assert_eq!(
            linear_insertion(Orientation::Horizontal, Point::new(5.0, 0.0), items),
            Some(LinearInsertion::Before(1))
        );
        assert_eq!(
            linear_insertion(Orientation::Horizontal, Point::new(70.0, 0.0), items),
            Some(LinearInsertion::After(2))
        );
    }

    #[test]
    fn midpoint_is_after_target() {
        let items = [(1, rect(10.0, 0.0, 20.0, 10.0))];

        assert_eq!(
            linear_insertion(Orientation::Horizontal, Point::new(19.0, 0.0), items),
            Some(LinearInsertion::Before(1))
        );
        assert_eq!(
            linear_insertion(Orientation::Horizontal, Point::new(20.0, 0.0), items),
            Some(LinearInsertion::After(1))
        );
    }

    #[test]
    fn gap_midpoint_prefers_previous_item() {
        let items = [
            (1, rect(0.0, 0.0, 10.0, 10.0)),
            (2, rect(20.0, 0.0, 10.0, 10.0)),
        ];

        assert_eq!(
            linear_insertion(Orientation::Horizontal, Point::new(15.0, 0.0), items),
            Some(LinearInsertion::After(1))
        );
        assert_eq!(
            linear_insertion(Orientation::Horizontal, Point::new(16.0, 0.0), items),
            Some(LinearInsertion::Before(2))
        );
    }

    #[test]
    fn orientation_controls_main_axis() {
        let items = [(1, rect(0.0, 10.0, 10.0, 20.0))];

        assert_eq!(
            linear_insertion(Orientation::Vertical, Point::new(0.0, 19.0), items),
            Some(LinearInsertion::Before(1))
        );
        assert_eq!(
            linear_insertion(Orientation::Vertical, Point::new(0.0, 20.0), items),
            Some(LinearInsertion::After(1))
        );
    }
}

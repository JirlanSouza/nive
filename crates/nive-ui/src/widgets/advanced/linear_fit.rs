/// Priority-aware linear overflow fit algorithm for widget-owned item data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearFitItem {
    /// Main-axis extent consumed by the item.
    pub extent: f32,
    /// Caller-defined priority. Higher values are kept before lower values.
    pub priority: i32,
}

impl LinearFitItem {
    /// Creates a fit item from an extent and priority.
    pub const fn new(extent: f32, priority: i32) -> Self {
        Self { extent, priority }
    }
}

/// Result of a [`LinearFit`] run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearFitResult {
    /// Original indexes that remain visible, in input order.
    pub visible: Vec<usize>,
    /// Original indexes that collapsed, in input order.
    pub collapsed: Vec<usize>,
}

/// Stateless priority-aware linear fit helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LinearFit;

impl LinearFit {
    /// Computes which items fit in `available_extent`.
    ///
    /// Lower-priority items collapse first. Equal-priority items collapse from
    /// the end of the input sequence first. At least one highest-priority item
    /// remains visible when input is non-empty, even if it exceeds the
    /// available extent on its own.
    pub fn fit(items: &[LinearFitItem], available_extent: f32) -> LinearFitResult {
        let mut visible: Vec<usize> = (0..items.len()).collect();
        let mut total = total_extent(items, &visible);
        let available_extent = available_extent.max(0.0);

        while total > available_extent && visible.len() > 1 {
            let remove_position = visible
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    let left_item = &items[**left];
                    let right_item = &items[**right];

                    left_item
                        .priority
                        .cmp(&right_item.priority)
                        .then_with(|| right.cmp(left))
                })
                .map(|(position, _)| position)
                .expect("visible contains at least one item");
            let removed = visible.remove(remove_position);

            total -= items[removed].extent.max(0.0);
        }

        let mut collapsed: Vec<usize> = (0..items.len())
            .filter(|index| !visible.contains(index))
            .collect();
        visible.sort_unstable();
        collapsed.sort_unstable();

        LinearFitResult { visible, collapsed }
    }
}

fn total_extent(items: &[LinearFitItem], indexes: &[usize]) -> f32 {
    indexes
        .iter()
        .map(|index| items[*index].extent.max(0.0))
        .sum()
}

#[cfg(test)]
mod linear_fit_tests {
    use super::*;

    #[test]
    fn all_items_fit() {
        let result = LinearFit::fit(
            &[
                LinearFitItem::new(20.0, 0),
                LinearFitItem::new(30.0, 0),
                LinearFitItem::new(40.0, 0),
            ],
            90.0,
        );

        assert_eq!(result.visible, vec![0, 1, 2]);
        assert!(result.collapsed.is_empty());
    }

    #[test]
    fn lower_priority_items_collapse_first() {
        let result = LinearFit::fit(
            &[
                LinearFitItem::new(40.0, 10),
                LinearFitItem::new(40.0, 1),
                LinearFitItem::new(40.0, 5),
            ],
            80.0,
        );

        assert_eq!(result.visible, vec![0, 2]);
        assert_eq!(result.collapsed, vec![1]);
    }

    #[test]
    fn equal_priorities_collapse_from_end_stably() {
        let items = [
            LinearFitItem::new(30.0, 1),
            LinearFitItem::new(30.0, 1),
            LinearFitItem::new(30.0, 1),
            LinearFitItem::new(30.0, 1),
        ];

        let first = LinearFit::fit(&items, 60.0);
        let second = LinearFit::fit(&items, 60.0);

        assert_eq!(first, second);
        assert_eq!(first.visible, vec![0, 1]);
        assert_eq!(first.collapsed, vec![2, 3]);
    }

    #[test]
    fn oversized_high_priority_item_remains_visible() {
        let result = LinearFit::fit(
            &[
                LinearFitItem::new(120.0, 10),
                LinearFitItem::new(20.0, 1),
                LinearFitItem::new(20.0, 1),
            ],
            80.0,
        );

        assert_eq!(result.visible, vec![0]);
        assert_eq!(result.collapsed, vec![1, 2]);
    }

    #[test]
    fn result_indexes_map_to_original_input() {
        let result = LinearFit::fit(
            &[
                LinearFitItem::new(25.0, 3),
                LinearFitItem::new(25.0, 1),
                LinearFitItem::new(25.0, 3),
                LinearFitItem::new(25.0, 2),
            ],
            75.0,
        );

        assert_eq!(result.visible, vec![0, 2, 3]);
        assert_eq!(result.collapsed, vec![1]);
    }
}

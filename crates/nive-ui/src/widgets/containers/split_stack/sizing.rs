/// How a [`SplitStack`] pane reacts to its container's main length.
///
/// [`SplitStack`]: super::SplitStack
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitSizing {
    /// Keeps this logical-pixel size; container growth leaves it untouched.
    Fixed(f32),
    /// Absorbs the space the fixed panes leave over.
    Fill,
}

pub(super) fn normalize_size(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

pub(super) fn pane_minimum(minimums: &[f32], index: usize) -> f32 {
    minimums.get(index).copied().map_or(0.0, normalize_size)
}

/// Index of the pane that absorbs the remainder.
///
/// Falls back to the last pane when none declares [`SplitSizing::Fill`], and
/// keeps only the first when several do.
pub(super) fn fill_index(sizing: &[SplitSizing]) -> Option<usize> {
    if sizing.is_empty() {
        return None;
    }

    sizing
        .iter()
        .position(|pane| matches!(pane, SplitSizing::Fill))
        .or(Some(sizing.len() - 1))
}

/// Resolves every pane length along the main axis into `out`.
///
/// Fixed panes are seeded at their requested size, the remainder pane takes what
/// is left, and a container too small for that makes fixed panes yield in
/// reverse order before falling back to a minimum-proportional allocation.
pub(super) fn resolve_into(
    sizing: &[SplitSizing],
    minimums: &[f32],
    available: f32,
    out: &mut Vec<f32>,
) {
    out.clear();

    let Some(fill) = fill_index(sizing) else {
        return;
    };
    let available = normalize_size(available);

    for (index, pane) in sizing.iter().enumerate() {
        let minimum = pane_minimum(minimums, index);
        out.push(match pane {
            SplitSizing::Fixed(size) => normalize_size(*size).max(minimum),
            SplitSizing::Fill => minimum,
        });
    }

    let mut deficit = pane_minimum(minimums, fill) - (available - fixed_total(out, fill));
    if deficit > 0.0 {
        for index in (0..out.len()).rev() {
            if deficit <= 0.0 {
                break;
            }
            if index == fill {
                continue;
            }

            let give = deficit
                .min(out[index] - pane_minimum(minimums, index))
                .max(0.0);
            out[index] -= give;
            deficit -= give;
        }
    }

    if deficit > 0.0 {
        allocate_by_minimum(minimums, available, out);
        return;
    }

    out[fill] = (available - fixed_total(out, fill)).max(0.0);
}

/// Applies a divider drag, clamped to what its two adjacent panes can give.
///
/// Returns the new leading and trailing lengths. Panes outside the adjacent pair
/// are never read or written, which is what keeps them exactly unchanged.
pub(super) fn resize(
    sizes: &[f32],
    minimums: &[f32],
    divider: usize,
    delta: f32,
) -> Option<(f32, f32)> {
    let leading = *sizes.get(divider)?;
    let trailing = *sizes.get(divider + 1)?;
    let leading_slack = (leading - pane_minimum(minimums, divider)).max(0.0);
    let trailing_slack = (trailing - pane_minimum(minimums, divider + 1)).max(0.0);
    let delta = if delta.is_finite() { delta } else { 0.0 };
    let delta = delta.clamp(-leading_slack, trailing_slack);

    Some((leading + delta, trailing - delta))
}

/// Signed distance a requested delta exceeded what the adjacent panes could give.
///
/// Negative went past the minimum of pane `divider`, positive past `divider + 1`.
/// Zero while either pane still has slack to absorb the move.
pub(super) fn overtravel(sizes: &[f32], minimums: &[f32], divider: usize, delta: f32) -> f32 {
    let Some(leading) = sizes.get(divider) else {
        return 0.0;
    };
    let Some(trailing) = sizes.get(divider + 1) else {
        return 0.0;
    };
    if !delta.is_finite() {
        return 0.0;
    }

    let leading_slack = (leading - pane_minimum(minimums, divider)).max(0.0);
    let trailing_slack = (trailing - pane_minimum(minimums, divider + 1)).max(0.0);

    if delta > trailing_slack {
        delta - trailing_slack
    } else if delta < -leading_slack {
        delta + leading_slack
    } else {
        0.0
    }
}

fn fixed_total(sizes: &[f32], fill: usize) -> f32 {
    sizes
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != fill)
        .map(|(_, size)| *size)
        .sum()
}

fn allocate_by_minimum(minimums: &[f32], available: f32, out: &mut [f32]) {
    let total: f32 = (0..out.len())
        .map(|index| pane_minimum(minimums, index))
        .sum();

    if total > 0.0 {
        for (index, size) in out.iter_mut().enumerate() {
            *size = available * pane_minimum(minimums, index) / total;
        }
    } else {
        let share = available / out.len() as f32;
        out.iter_mut().for_each(|size| *size = share);
    }
}

#[cfg(test)]
mod split_stack_sizing_tests {
    use super::*;

    const SIDES: [SplitSizing; 3] = [
        SplitSizing::Fixed(280.0),
        SplitSizing::Fill,
        SplitSizing::Fixed(320.0),
    ];
    const MINIMUMS: [f32; 3] = [160.0, 240.0, 160.0];

    fn resolved(sizing: &[SplitSizing], minimums: &[f32], available: f32) -> Vec<f32> {
        let mut out = Vec::new();
        resolve_into(sizing, minimums, available, &mut out);
        out
    }

    #[test]
    fn fill_pane_absorbs_container_growth() {
        assert_eq!(
            resolved(&SIDES, &MINIMUMS, 1400.0),
            vec![280.0, 800.0, 320.0]
        );
        assert_eq!(
            resolved(&SIDES, &MINIMUMS, 1920.0),
            vec![280.0, 1320.0, 320.0]
        );
    }

    #[test]
    fn fixed_panes_yield_in_reverse_order_when_the_container_shrinks() {
        // 780 leaves the fill pane 180 against a 240 minimum, so the last fixed
        // pane covers the 60 shortfall alone and the first one is untouched.
        assert_eq!(
            resolved(&SIDES, &MINIMUMS, 780.0),
            vec![280.0, 240.0, 260.0]
        );
        // 700 still fits inside the last pane's 160 of slack.
        assert_eq!(
            resolved(&SIDES, &MINIMUMS, 700.0),
            vec![280.0, 240.0, 180.0]
        );
        // 600 exhausts it, so the shortfall reaches the first fixed pane.
        assert_eq!(
            resolved(&SIDES, &MINIMUMS, 600.0),
            vec![200.0, 240.0, 160.0]
        );
    }

    #[test]
    fn an_impossible_container_allocates_proportionally_to_minimums() {
        let sizes = resolved(&SIDES, &MINIMUMS, 280.0);

        // Minimums total 560, so each pane receives half of its own.
        assert_eq!(sizes, vec![80.0, 120.0, 80.0]);
        assert_eq!(sizes.iter().sum::<f32>(), 280.0);
    }

    #[test]
    fn fixed_panes_never_render_under_their_minimum() {
        assert_eq!(
            resolved(
                &[SplitSizing::Fixed(10.0), SplitSizing::Fill],
                &[160.0, 240.0],
                1000.0
            ),
            vec![160.0, 840.0]
        );
    }

    #[test]
    fn fill_index_normalizes_missing_and_repeated_declarations() {
        assert_eq!(fill_index(&SIDES), Some(1));
        assert_eq!(
            fill_index(&[SplitSizing::Fixed(10.0), SplitSizing::Fixed(20.0)]),
            Some(1)
        );
        assert_eq!(fill_index(&[SplitSizing::Fill, SplitSizing::Fill]), Some(0));
        assert_eq!(fill_index(&[]), None);
    }

    #[test]
    fn repeated_fill_declarations_collapse_to_their_minimums() {
        assert_eq!(
            resolved(
                &[SplitSizing::Fill, SplitSizing::Fill],
                &[100.0, 150.0],
                600.0
            ),
            vec![450.0, 150.0]
        );
    }

    #[test]
    fn non_finite_input_resolves_finitely() {
        let sizing = [SplitSizing::Fixed(f32::NAN), SplitSizing::Fill];
        let sizes = resolved(&sizing, &[f32::INFINITY, -5.0], 500.0);

        assert_eq!(sizes, vec![0.0, 500.0]);
        // A non-finite container length resolves as zero, so nothing renders.
        assert_eq!(resolved(&sizing, &MINIMUMS, f32::NAN), vec![0.0, 0.0]);
        assert!(resolved(&SIDES, &MINIMUMS, 1400.0)
            .iter()
            .all(|s| s.is_finite()));
    }

    #[test]
    fn resize_moves_the_adjacent_pair_one_to_one() {
        let sizes = [280.0, 800.0, 320.0];

        assert_eq!(resize(&sizes, &MINIMUMS, 0, 120.0), Some((400.0, 680.0)));
        assert_eq!(resize(&sizes, &MINIMUMS, 1, -200.0), Some((600.0, 520.0)));
    }

    #[test]
    fn resize_stops_at_the_neighbour_minimum_without_spilling() {
        let sizes = [280.0, 800.0, 320.0];

        // The centre can only give 560 before hitting its 240 minimum.
        assert_eq!(resize(&sizes, &MINIMUMS, 0, 5_000.0), Some((840.0, 240.0)));
        // The left pane can only give 120 before hitting its 160 minimum.
        assert_eq!(resize(&sizes, &MINIMUMS, 0, -5_000.0), Some((160.0, 920.0)));
    }

    #[test]
    fn resize_leaves_every_non_adjacent_pane_untouched() {
        let sizes = [280.0, 800.0, 320.0];

        for step in 0..=200 {
            let delta = -1_000.0 + 10.0 * step as f32;
            let (leading, trailing) = resize(&sizes, &MINIMUMS, 1, delta).expect("divider 1");

            // Divider 1 borders panes 1 and 2, so pane 0 is outside the pair and
            // its length is never part of the computation.
            assert_eq!(leading + trailing, sizes[1] + sizes[2]);
            assert!(leading >= MINIMUMS[1] && trailing >= MINIMUMS[2]);
        }
    }

    #[test]
    fn overtravel_reports_only_what_the_clamp_discarded() {
        let sizes = [280.0, 800.0, 320.0];

        // Inside the slack of either neighbour, nothing is discarded.
        assert_eq!(overtravel(&sizes, &MINIMUMS, 0, 120.0), 0.0);
        assert_eq!(overtravel(&sizes, &MINIMUMS, 0, -120.0), 0.0);
        // The centre can give 560, the left pane 120.
        assert_eq!(overtravel(&sizes, &MINIMUMS, 0, 560.0), 0.0);
        assert_eq!(overtravel(&sizes, &MINIMUMS, 0, 600.0), 40.0);
        assert_eq!(overtravel(&sizes, &MINIMUMS, 0, -160.0), -40.0);
    }

    #[test]
    fn overtravel_is_finite_for_degenerate_input() {
        let sizes = [280.0, 800.0, 320.0];

        assert_eq!(overtravel(&sizes, &MINIMUMS, 0, f32::NAN), 0.0);
        assert_eq!(overtravel(&sizes, &MINIMUMS, 9, 500.0), 0.0);
        assert_eq!(overtravel(&sizes, &MINIMUMS, 2, 500.0), 0.0);
        assert_eq!(overtravel(&sizes, &[f32::NAN; 3], 0, 5_000.0), 4_200.0);
    }

    #[test]
    fn resize_rejects_a_divider_past_the_last_pair() {
        let sizes = [280.0, 800.0, 320.0];

        assert_eq!(resize(&sizes, &MINIMUMS, 2, 10.0), None);
        assert_eq!(resize(&sizes, &MINIMUMS, 9, 10.0), None);
        assert_eq!(resize(&sizes, &MINIMUMS, 0, f32::NAN), Some((280.0, 800.0)));
    }
}

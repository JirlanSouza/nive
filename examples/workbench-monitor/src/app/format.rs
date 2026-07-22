//! App-side digit grouping for numeric content. `MetricCard` and inline text
//! widgets already accept preformatted values, so grouping conventions stay
//! out of the framework and live here instead.

/// Groups a non-negative integer's digits with a thousands separator.
pub(super) fn grouped(value: impl std::fmt::Display) -> String {
    let raw = value.to_string();
    let (sign, digits) = raw
        .strip_prefix('-')
        .map_or(("", raw.as_str()), |digits| ("-", digits));

    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().rev().enumerate() {
        if index != 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }

    format!("{sign}{}", grouped.chars().rev().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_values_above_the_first_boundary() {
        assert_eq!(grouped(18_420), "18,420");
        assert_eq!(grouped(1_000_000), "1,000,000");
    }

    #[test]
    fn boundary_just_below_the_first_separator_stays_ungrouped() {
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
    }

    #[test]
    fn small_values_are_unchanged() {
        assert_eq!(grouped(43), "43");
        assert_eq!(grouped(0), "0");
    }
}

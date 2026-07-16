use crate::theme::{self, ControlSize, TextStyle, TypographyRole};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetadataMetrics {
    pub list_gap: f32,
    pub slot_gap: f32,
    pub minimum_height: f32,
    pub text_style: TextStyle,
}

pub fn metrics(size: ControlSize) -> MetadataMetrics {
    let theme = theme::active();
    let control = theme.control_metrics(size);
    let spacing = theme.spacing();

    MetadataMetrics {
        list_gap: match size {
            ControlSize::Xs => spacing.xxs,
            ControlSize::Sm => spacing.xs,
            ControlSize::Md => spacing.sm,
            ControlSize::Lg => spacing.md,
        },
        slot_gap: control.gap,
        minimum_height: control.height,
        text_style: theme.typography(TypographyRole::Body),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{testing::ThemeTestGuard, Theme, ThemeDensity, ThemeMode};

    #[test]
    fn metadata_text_hierarchy_stays_complete_14px_across_sizes_and_density() {
        for density in [
            ThemeDensity::Compact,
            ThemeDensity::Standard,
            ThemeDensity::Comfortable,
        ] {
            let name = match density {
                ThemeDensity::Compact => "Metadata Compact",
                ThemeDensity::Standard => "Metadata Standard",
                ThemeDensity::Comfortable => "Metadata Comfortable",
            };
            let _guard = ThemeTestGuard::activate(
                Theme::builder(name, ThemeMode::Light)
                    .density(density)
                    .build(),
            );
            for size in [
                ControlSize::Xs,
                ControlSize::Sm,
                ControlSize::Md,
                ControlSize::Lg,
            ] {
                assert_eq!(metrics(size).text_style.size, 14.0);
                assert_eq!(
                    metrics(size).minimum_height,
                    Theme::builder("Expected", ThemeMode::Light)
                        .density(density)
                        .build()
                        .control_metrics(size)
                        .height
                );
            }
        }
    }
}

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ThemeDensity {
    Comfortable,
    #[default]
    Standard,
    Compact,
}

impl ThemeDensity {
    pub const ALL: [Self; 3] = [Self::Comfortable, Self::Standard, Self::Compact];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Comfortable => "Comfortable",
            Self::Standard => "Standard",
            Self::Compact => "Compact",
        }
    }
}

impl fmt::Display for ThemeDensity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod density_tests {
    use super::*;

    #[test]
    fn standard_is_default() {
        assert_eq!(ThemeDensity::default(), ThemeDensity::Standard);
    }

    #[test]
    fn all_variants_are_unique() {
        let mut names: Vec<_> = ThemeDensity::ALL.iter().map(|d| d.name()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), ThemeDensity::ALL.len());
    }
}

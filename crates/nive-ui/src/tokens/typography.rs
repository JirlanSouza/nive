use iced::font::{Family, Font, Weight};

pub struct FontFamily {
    name: &'static str,
}

impl FontFamily {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub fn with_weight(&self, weight: Weight) -> Font {
        Font {
            family: Family::Name(self.name),
            weight,
            ..Font::DEFAULT
        }
    }

    pub fn thin(&self) -> Font {
        self.with_weight(Weight::Thin)
    }

    pub fn light(&self) -> Font {
        self.with_weight(Weight::Light)
    }

    pub fn normal(&self) -> Font {
        self.with_weight(Weight::Normal)
    }

    pub fn medium(&self) -> Font {
        self.with_weight(Weight::Medium)
    }

    pub fn semibold(&self) -> Font {
        self.with_weight(Weight::Semibold)
    }

    pub fn bold(&self) -> Font {
        self.with_weight(Weight::Bold)
    }

    pub fn black(&self) -> Font {
        self.with_weight(Weight::Black)
    }
}

pub const UI: FontFamily = FontFamily::new("Inter");
pub const MONO: FontFamily = FontFamily::new("Geist Mono");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSize {
    Xs,
    Sm,
    Base,
    Lg,
    Xl,
    Xxl,
    Xxxl,
    Xxxxl,
}

impl TextSize {
    pub const fn value(self) -> f32 {
        match self {
            Self::Xs => TEXT_XS,
            Self::Sm => TEXT_SM,
            Self::Base => TEXT_BASE,
            Self::Lg => TEXT_LG,
            Self::Xl => TEXT_XL,
            Self::Xxl => TEXT_2XL,
            Self::Xxxl => TEXT_3XL,
            Self::Xxxxl => TEXT_4XL,
        }
    }
}

pub const TEXT_XS: f32 = 12.0;
pub const TEXT_SM: f32 = 14.0;
pub const TEXT_BASE: f32 = 14.0;
pub const TEXT_LG: f32 = 16.0;
pub const TEXT_XL: f32 = 20.0;
pub const TEXT_2XL: f32 = 24.0;
pub const TEXT_3XL: f32 = 30.0;
pub const TEXT_4XL: f32 = 36.0;

pub const LEADING_NONE: f32 = 1.0;
pub const LEADING_TIGHT: f32 = 1.25;
pub const LEADING_SNUG: f32 = 1.375;
pub const LEADING_NORMAL: f32 = 1.5;
pub const LEADING_RELAXED: f32 = 1.625;

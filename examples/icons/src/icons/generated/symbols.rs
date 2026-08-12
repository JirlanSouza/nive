// Bundled Lucide icons are sourced from Lucide (https://lucide.dev) and distributed under the ISC License.
use nive::prelude::{IconRef, IconSource};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconSymbol {
    BrandMark,
}

impl IconSource for IconSymbol {
    fn svg_bytes(self) -> &'static [u8] {
        match self {
            Self::BrandMark => {
                include_bytes!("../../../assets/icons/generated/custom/brand-mark.svg")
            }
        }
    }

    fn provider_slug(self) -> &'static str {
        match self {
            Self::BrandMark => "custom:brand-mark",
        }
    }
}

impl From<IconSymbol> for IconRef {
    fn from(symbol: IconSymbol) -> Self {
        Self::from_source(symbol)
    }
}

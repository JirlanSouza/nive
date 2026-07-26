// Bundled Lucide icons are sourced from Lucide (https://lucide.dev) and distributed under the ISC License.
use crate::icons::{IconRef, IconSource};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconSymbol {}

impl IconSource for IconSymbol {
    fn svg_bytes(self) -> &'static [u8] {
        match self {}
    }

    fn provider_slug(self) -> &'static str {
        match self {}
    }
}

impl From<IconSymbol> for IconRef {
    fn from(symbol: IconSymbol) -> Self {
        Self::from_source(symbol)
    }
}

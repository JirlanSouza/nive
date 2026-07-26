// Bundled Lucide icons are sourced from Lucide (https://lucide.dev) and distributed under the ISC License.
use nive::prelude::{IconRef, IconSource};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconSymbol {
    Dashboard,
    Dependency,
    Diagnostics,
    Event,
    Job,
    Server,
}

impl IconSource for IconSymbol {
    fn svg_bytes(self) -> &'static [u8] {
        match self {
            Self::Dashboard => include_bytes!("../../../assets/icons/generated/lucide/layout-dashboard.svg"),
            Self::Dependency => include_bytes!("../../../assets/icons/generated/lucide/network.svg"),
            Self::Diagnostics => include_bytes!("../../../assets/icons/generated/lucide/stethoscope.svg"),
            Self::Event => include_bytes!("../../../assets/icons/generated/lucide/zap.svg"),
            Self::Job => include_bytes!("../../../assets/icons/generated/lucide/play.svg"),
            Self::Server => include_bytes!("../../../assets/icons/generated/lucide/server.svg"),
        }
    }

    fn provider_slug(self) -> &'static str {
        match self {
            Self::Dashboard => "lucide:layout-dashboard",
            Self::Dependency => "lucide:network",
            Self::Diagnostics => "lucide:stethoscope",
            Self::Event => "lucide:zap",
            Self::Job => "lucide:play",
            Self::Server => "lucide:server",
        }
    }
}

impl From<IconSymbol> for IconRef {
    fn from(symbol: IconSymbol) -> Self {
        Self::from_source(symbol)
    }
}

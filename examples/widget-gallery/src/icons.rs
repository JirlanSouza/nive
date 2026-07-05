use nive::prelude::{IconCatalog, IconCatalogEntry, IconGlyph, IconRole};

pub const APP_ICON_CATALOG: IconCatalog = IconCatalog::new(&[IconCatalogEntry::new(
    IconRole::WindowClose,
    IconGlyph::new(
        include_bytes!("../assets/icons/gallery-symbol.svg"),
        "custom:gallery-symbol",
    ),
)]);

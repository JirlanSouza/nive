use nive::prelude::{IconCatalog, IconCatalogEntry, IconGlyph, IconRole};

const GALLERY_SYMBOL: IconGlyph = IconGlyph::new(
    include_bytes!("../assets/icons/gallery-symbol.svg"),
    "custom:gallery-symbol",
);

pub const APP_ICON_CATALOG: IconCatalog = IconCatalog::new(&[
    IconCatalogEntry::new(IconRole::WindowClose, GALLERY_SYMBOL),
    IconCatalogEntry::new(IconRole::Identity, GALLERY_SYMBOL),
]);

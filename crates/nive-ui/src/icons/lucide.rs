use super::{
    default_catalog as generated_default_catalog, default_glyph_for, IconCatalog, IconGlyph,
    IconRole,
};

pub const fn default_catalog() -> IconCatalog {
    generated_default_catalog()
}

pub fn glyph_for(role: IconRole) -> IconGlyph {
    default_glyph_for(role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_covers_all_roles() {
        for role in IconRole::ALL {
            assert!(
                default_catalog().covers(*role),
                "missing default glyph for {}",
                role.canonical_name()
            );
        }
    }

    #[test]
    fn default_catalog_covers_identity() {
        let glyph = glyph_for(IconRole::Identity);

        assert_eq!(glyph.provider_slug(), "lucide:user");
    }
}

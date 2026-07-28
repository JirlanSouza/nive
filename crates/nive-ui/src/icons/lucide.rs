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

    /// `default_glyph_for` unwraps its lookup, so a role missing from
    /// `icons.toml` panics in every app that renders it. No tool can check this:
    /// `nive-cli` generates the catalog but does not know what `IconRole`
    /// declares. On failure, add the role and run `just icons-sync`.
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

    #[test]
    fn default_catalog_maps_validation_error_without_losing_identity() {
        assert_eq!(
            glyph_for(IconRole::ValidationError).provider_slug(),
            "lucide:circle-alert"
        );
        assert_eq!(glyph_for(IconRole::Identity).provider_slug(), "lucide:user");
    }
}

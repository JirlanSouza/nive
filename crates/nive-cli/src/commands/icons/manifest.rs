use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use super::generate::write_if_changed;
use super::{IconPaths, IconsManifest, LucideProviderConfig, ProviderConfig, ProviderRef, Result};

impl Default for LucideProviderConfig {
    fn default() -> Self {
        Self {
            version: "0.460.0".to_string(),
            stroke_width: "2".to_string(),
            stroke_linecap: "round".to_string(),
            stroke_linejoin: "round".to_string(),
        }
    }
}

impl ProviderRef {
    pub(super) fn parse(value: &str) -> Result<Self> {
        let Some((provider, name)) = value.split_once(':') else {
            return Err(format!(
                "Icon ref `{value}` is missing a provider. Use `lucide:{value}` or `custom:<name>`."
            )
            .into());
        };

        match provider {
            "lucide" => {
                validate_lucide_slug(name)?;
                Ok(Self::Lucide(name.to_string()))
            }
            "custom" => {
                validate_ref_name(name)?;
                Ok(Self::Custom(name.to_string()))
            }
            _ => Err(format!(
                "Unknown icon provider `{provider}` in `{value}`. Supported providers: lucide, custom."
            )
            .into()),
        }
    }

    pub(super) fn parse_command_input(value: &str) -> Result<Self> {
        if value.contains(':') {
            return Self::parse(value);
        }

        validate_lucide_slug(value)?;
        Ok(Self::Lucide(value.to_string()))
    }

    pub(super) fn normalized(&self) -> String {
        match self {
            Self::Lucide(slug) => format!("lucide:{slug}"),
            Self::Custom(name) => format!("custom:{name}"),
        }
    }

    pub(super) fn provider_slug(&self) -> String {
        self.normalized()
    }

    pub(super) fn generated_asset_path(&self) -> PathBuf {
        match self {
            Self::Lucide(slug) => PathBuf::from("lucide").join(format!("{slug}.svg")),
            Self::Custom(name) => PathBuf::from("custom").join(format!("{name}.svg")),
        }
    }
}

impl IconPaths {
    pub(super) fn from_root(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            manifest: root.join("icons.toml"),
            generated_asset_dir: root.join("assets/icons/generated"),
            icons_rs: root.join("src/icons.rs"),
            generated_rs: root.join("src/icons/generated.rs"),
            generated_catalog: root.join("src/icons/generated/catalog.rs"),
            generated_symbols: root.join("src/icons/generated/symbols.rs"),
            gallery_dir: root.join("target/nive/icons"),
            metadata_cache_dir: root.join("target/nive/icons/cache"),
            legacy_manifest: root.join("icons/lucide.toml"),
        }
    }
}

/// A manifest that declares nothing but its provider.
///
/// Roles start empty on purpose: an application's manifest is additive over the
/// framework catalog, so an app that overrides nothing declares nothing. The
/// framework's own full mapping lives in `crates/nive-ui/icons.toml`.
pub(super) fn empty_manifest() -> IconsManifest {
    IconsManifest {
        provider: ProviderConfig::default(),
        roles: BTreeMap::new(),
        symbols: BTreeMap::new(),
        custom: BTreeMap::new(),
    }
}

pub(super) fn require_manifest(paths: &IconPaths) -> Result<()> {
    if paths.manifest.exists() {
        return Ok(());
    }

    if paths.legacy_manifest.exists() {
        return Err(format!(
            "No `icons.toml` found at {}. Found legacy {} instead; migrate to app-root `icons.toml` with [provider.lucide], [roles], [symbols], and [custom].",
            paths.manifest.display(),
            paths.legacy_manifest.display()
        )
        .into());
    }

    Err(format!(
        "No `icons.toml` found at {}. Run `nive icons init` first.",
        paths.manifest.display()
    )
    .into())
}

pub(super) fn read_manifest(path: &Path) -> Result<IconsManifest> {
    let contents = fs::read_to_string(path)?;
    let manifest: IconsManifest = toml::from_str(&contents)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub(super) fn read_custom_svg(
    paths: &IconPaths,
    manifest: &IconsManifest,
    name: &str,
) -> Result<String> {
    let Some(relative) = manifest.custom.get(name) else {
        return Err(format!("Custom icon `{name}` is not registered in [custom].").into());
    };

    let path = validate_custom_svg_path(paths, relative)?;
    fs::read_to_string(path).map_err(Into::into)
}

pub(super) fn write_manifest(paths: &IconPaths, manifest: &IconsManifest) -> Result<()> {
    let updated = toml::to_string_pretty(manifest)?;
    write_if_changed(&paths.manifest, updated.as_bytes())?;
    Ok(())
}

mod validate;

pub(super) use validate::*;

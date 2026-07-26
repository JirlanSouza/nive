use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
};

use super::generate::write_if_changed;
use super::lucide::display_path;
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

pub(super) fn empty_manifest() -> IconsManifest {
    IconsManifest {
        provider: ProviderConfig::default(),
        roles: default_role_refs(),
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

pub(super) fn validate_manifest(manifest: &IconsManifest) -> Result<()> {
    validate_lucide_config(&manifest.provider.lucide)?;

    for (role_name, value) in &manifest.roles {
        validate_role_name(role_name)?;
        validate_manifest_ref(value)?;
    }

    for (variant, value) in &manifest.symbols {
        validate_variant(variant)?;
        validate_manifest_ref(value)?;
    }

    for (name, path) in &manifest.custom {
        validate_ref_name(name)?;
        validate_manifest_relative_svg_path(path)?;
    }

    for value in manifest.roles.values().chain(manifest.symbols.values()) {
        validate_ref_custom_target(manifest, &ProviderRef::parse(value)?)?;
    }

    Ok(())
}

pub(super) fn validate_lucide_config(config: &LucideProviderConfig) -> Result<()> {
    if config.version.trim().is_empty() {
        return Err("Lucide provider version cannot be empty".into());
    }
    if config.stroke_width.trim().is_empty() {
        return Err("Lucide stroke_width cannot be empty".into());
    }

    validate_stroke_line_value("stroke_linecap", &config.stroke_linecap)?;
    validate_stroke_line_value("stroke_linejoin", &config.stroke_linejoin)?;
    Ok(())
}

pub(super) fn validate_manifest_ref(value: &str) -> Result<()> {
    ProviderRef::parse(value).map(|_| ())
}

pub(super) fn validate_ref_custom_target(
    manifest: &IconsManifest,
    icon_ref: &ProviderRef,
) -> Result<()> {
    if let ProviderRef::Custom(name) = icon_ref {
        if !manifest.custom.contains_key(name) {
            return Err(format!(
                "Icon ref `custom:{name}` is not registered in [custom]. Add it with `nive icons add-custom {name} <path>`."
            )
            .into());
        }
    }

    Ok(())
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

pub(super) fn validate_custom_svg_path(paths: &IconPaths, relative: &str) -> Result<PathBuf> {
    validate_manifest_relative_svg_path(relative)?;

    let root = fs::canonicalize(&paths.root)?;
    let path = paths
        .manifest
        .parent()
        .unwrap_or(&paths.root)
        .join(relative);
    let path = fs::canonicalize(&path).map_err(|error| {
        format!(
            "{} is not readable from {} (error: {error})",
            relative,
            display_path(&paths.manifest)
        )
    })?;

    if !path.starts_with(&root) {
        return Err(format!(
            "{} escapes the app crate root {}",
            display_path(&path),
            display_path(&root)
        )
        .into());
    }

    let contents = fs::read_to_string(&path)?;
    if !contents.trim_start().starts_with("<svg") && !contents.contains("<svg") {
        return Err(format!("{} does not contain an SVG root", display_path(&path)).into());
    }
    validate_custom_svg_contract(&contents).map_err(|error| {
        format!(
            "{} violates the Nive custom icon contract: {error}",
            display_path(&path)
        )
    })?;

    Ok(path)
}

pub(super) fn validate_custom_svg_contract(source: &str) -> Result<()> {
    let start = source.find("<svg").ok_or("SVG root not found")?;
    let root_end = source[start..]
        .find('>')
        .map(|index| start + index)
        .ok_or("SVG root is not closed")?;
    let root = &source[start..=root_end];

    for (attribute, value) in [
        ("viewBox", "0 0 24 24"),
        ("fill", "none"),
        ("stroke", "currentColor"),
        ("stroke-width", "2"),
        ("stroke-linecap", "round"),
        ("stroke-linejoin", "round"),
    ] {
        let double_quoted = format!(r#"{attribute}="{value}""#);
        let single_quoted = format!("{attribute}='{value}'");
        if !root.contains(&double_quoted) && !root.contains(&single_quoted) {
            return Err(format!(
                "expected `{attribute}=\"{value}\"` on the SVG root (24×24, stroke-2, rounded, monochrome currentColor)"
            )
            .into());
        }
    }

    validate_monochrome_attribute(source, "fill", &["none", "currentColor"])?;
    validate_monochrome_attribute(source, "stroke", &["none", "currentColor"])?;
    validate_attribute_values(source, "stroke-width", &["2"])?;
    validate_attribute_values(source, "stroke-linecap", &["round"])?;
    validate_attribute_values(source, "stroke-linejoin", &["round"])?;

    Ok(())
}

pub(super) fn validate_monochrome_attribute(
    source: &str,
    attribute: &str,
    allowed: &[&str],
) -> Result<()> {
    validate_attribute_values(source, attribute, allowed).map_err(|_| {
        format!("`{attribute}` must be monochrome (`none` or `currentColor`), not a fixed paint")
            .into()
    })
}

pub(super) fn validate_attribute_values(
    source: &str,
    attribute: &str,
    allowed: &[&str],
) -> Result<()> {
    let needle = format!("{attribute}=");
    let mut remainder = source;

    while let Some(index) = remainder.find(&needle) {
        let after_equals = &remainder[index + needle.len()..];
        let Some(quote) = after_equals.chars().next() else {
            break;
        };
        if quote != '"' && quote != '\'' {
            return Err(format!("`{attribute}` must use a quoted value").into());
        }
        let value_start = quote.len_utf8();
        let value_tail = &after_equals[value_start..];
        let value_end = value_tail
            .find(quote)
            .ok_or_else(|| format!("`{attribute}` has an unterminated value"))?;
        let value = &value_tail[..value_end];
        if !allowed.contains(&value) {
            return Err(format!(
                "`{attribute}` must be one of {}, found `{value}`",
                allowed.join(", ")
            )
            .into());
        }
        remainder = &value_tail[value_end + quote.len_utf8()..];
    }

    Ok(())
}

pub(super) fn validate_manifest_relative_svg_path(relative: &str) -> Result<()> {
    let path = Path::new(relative);
    if path.is_absolute() {
        return Err(format!("Custom SVG path `{relative}` must be manifest-relative.").into());
    }

    if path.extension() != Some(OsStr::new("svg")) {
        return Err(format!("Custom SVG path `{relative}` must point to an .svg file.").into());
    }

    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(format!("Custom SVG path `{relative}` cannot contain `..`.").into());
    }

    Ok(())
}

pub(super) fn write_manifest(paths: &IconPaths, manifest: &IconsManifest) -> Result<()> {
    let updated = toml::to_string_pretty(manifest)?;
    write_if_changed(&paths.manifest, updated.as_bytes())?;
    Ok(())
}

pub(super) fn validate_role_name(name: &str) -> Result<()> {
    if role_variant(name).is_ok() {
        return Ok(());
    }

    Err(format!(
        "Unknown icon role `{name}`. Expected one of: {}",
        required_role_names().join(", ")
    )
    .into())
}

pub(super) fn validate_variant(variant: &str) -> Result<()> {
    let mut chars = variant.chars();
    let Some(first) = chars.next() else {
        return Err("Icon symbol variant cannot be empty".into());
    };

    if !first.is_ascii_uppercase() {
        return Err(format!(
            "Icon symbol variant must start with uppercase ASCII (variant: {variant})"
        )
        .into());
    }

    if !chars.all(|ch| ch.is_ascii_alphanumeric()) {
        return Err(
            format!("Icon symbol variant must be PascalCase ASCII (variant: {variant})").into(),
        );
    }

    Ok(())
}

pub(super) fn validate_lucide_slug(slug: &str) -> Result<()> {
    if slug.is_empty() {
        return Err("Lucide icon name cannot be empty".into());
    }

    if !slug
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(format!("Lucide icon name must be kebab-case ASCII (icon: {slug})").into());
    }

    Ok(())
}

pub(super) fn validate_ref_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err("Custom icon name cannot be empty".into());
    }

    if !name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(format!("Custom icon name must be kebab-case ASCII (name: {name})").into());
    }

    Ok(())
}

pub(super) fn validate_stroke_line_value(field: &str, value: &str) -> Result<()> {
    match value {
        "butt" | "round" | "square" | "arcs" | "bevel" | "miter" | "miter-clip" => Ok(()),
        _ => Err(format!("Invalid {field} value (value: {value})").into()),
    }
}

pub(super) fn ensure_provider(provider: &str) -> Result<()> {
    if provider == "lucide" {
        Ok(())
    } else {
        Err(format!("Unsupported icon provider `{provider}`. Supported providers: lucide.").into())
    }
}

pub(super) fn default_role_refs() -> BTreeMap<String, String> {
    role_lucide_defaults()
        .iter()
        .map(|(role, slug, _variant)| ((*role).to_string(), format!("lucide:{slug}")))
        .collect()
}

pub(super) fn required_role_names() -> Vec<&'static str> {
    role_lucide_defaults()
        .iter()
        .map(|(role, _slug, _variant)| *role)
        .collect()
}

pub(super) fn role_variant(role: &str) -> Result<&'static str> {
    role_lucide_defaults()
        .iter()
        .find(|(candidate, _slug, _variant)| *candidate == role)
        .map(|(_role, _slug, variant)| *variant)
        .ok_or_else(|| format!("Unknown icon role `{role}`.").into())
}

pub(super) fn role_lucide_defaults() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        ("action-confirm", "check", "ActionConfirm"),
        ("dialog-error", "circle-alert", "DialogError"),
        ("dialog-information", "info", "DialogInformation"),
        ("dialog-success", "circle-check", "DialogSuccess"),
        ("dialog-warning", "triangle-alert", "DialogWarning"),
        ("edit-copy", "copy", "EditCopy"),
        ("edit-delete", "trash", "EditDelete"),
        ("edit-find", "search", "EditFind"),
        ("edit-modify", "pencil", "EditModify"),
        ("folder", "folder", "Folder"),
        ("go-next", "arrow-right", "GoNext"),
        ("go-previous", "arrow-left", "GoPrevious"),
        ("identity", "user", "Identity"),
        ("list-add", "plus", "ListAdd"),
        ("list-remove", "minus", "ListRemove"),
        ("mail-inbox", "inbox", "MailInbox"),
        ("nive-disclosure-down", "chevron-down", "NiveDisclosureDown"),
        ("nive-disclosure-left", "chevron-left", "NiveDisclosureLeft"),
        (
            "nive-disclosure-right",
            "chevron-right",
            "NiveDisclosureRight",
        ),
        ("nive-disclosure-up", "chevron-up", "NiveDisclosureUp"),
        ("open-menu", "menu", "OpenMenu"),
        ("preferences-system", "settings", "PreferencesSystem"),
        ("tab-pinned", "pin", "TabPinned"),
        ("validation-error", "circle-alert", "ValidationError"),
        ("notification-alert", "bell", "NotificationAlert"),
        ("view-activity", "activity", "ViewActivity"),
        ("view-conceal", "eye-off", "ViewConceal"),
        ("view-maximize", "maximize-2", "ViewMaximize"),
        ("view-more", "ellipsis", "ViewMore"),
        ("view-refresh", "refresh-cw", "ViewRefresh"),
        ("view-restore", "minimize-2", "ViewRestore"),
        ("view-theme", "palette", "ViewTheme"),
        ("view-reveal", "eye", "ViewReveal"),
        ("window-close", "x", "WindowClose"),
    ]
}

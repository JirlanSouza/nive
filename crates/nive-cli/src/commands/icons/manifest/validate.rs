//! What a manifest has to satisfy before anything is generated from it.

use std::{
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
};

use super::super::lucide::display_path;
use super::super::{IconPaths, IconsManifest, LucideProviderConfig, ProviderRef, Result};

pub(in crate::commands::icons) fn validate_manifest(manifest: &IconsManifest) -> Result<()> {
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

pub(in crate::commands::icons) fn validate_lucide_config(
    config: &LucideProviderConfig,
) -> Result<()> {
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

pub(in crate::commands::icons) fn validate_manifest_ref(value: &str) -> Result<()> {
    ProviderRef::parse(value).map(|_| ())
}

pub(in crate::commands::icons) fn validate_ref_custom_target(
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
pub(in crate::commands::icons) fn validate_custom_svg_path(
    paths: &IconPaths,
    relative: &str,
) -> Result<PathBuf> {
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

pub(in crate::commands::icons) fn validate_custom_svg_contract(source: &str) -> Result<()> {
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

pub(in crate::commands::icons) fn validate_monochrome_attribute(
    source: &str,
    attribute: &str,
    allowed: &[&str],
) -> Result<()> {
    validate_attribute_values(source, attribute, allowed).map_err(|_| {
        format!("`{attribute}` must be monochrome (`none` or `currentColor`), not a fixed paint")
            .into()
    })
}

pub(in crate::commands::icons) fn validate_attribute_values(
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

pub(in crate::commands::icons) fn validate_manifest_relative_svg_path(
    relative: &str,
) -> Result<()> {
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
/// Checks that a role name is *spelled* like a role, not that it exists —
/// existence is the app compiler's question. A well-formed name guarantees the
/// variant [`role_variant`] derives is a legal Rust identifier.
pub(in crate::commands::icons) fn validate_role_name(name: &str) -> Result<()> {
    let malformed = |reason: &str| -> Result<()> {
        Err(format!(
            "Icon role `{name}` is not kebab-case ASCII ({reason}). \
             Roles are declared by `IconRole` in the `nive` version this project \
             depends on; see its API docs for the list."
        )
        .into())
    };

    let Some(first) = name.chars().next() else {
        return malformed("empty");
    };

    if !first.is_ascii_lowercase() {
        return malformed("must start with a lowercase ASCII letter");
    }

    if !name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return malformed("only lowercase ASCII letters, digits, and `-` are allowed");
    }

    if name.split('-').any(str::is_empty) {
        return malformed("segments cannot be empty");
    }

    Ok(())
}

pub(in crate::commands::icons) fn validate_variant(variant: &str) -> Result<()> {
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

pub(in crate::commands::icons) fn validate_lucide_slug(slug: &str) -> Result<()> {
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

pub(in crate::commands::icons) fn validate_ref_name(name: &str) -> Result<()> {
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

pub(in crate::commands::icons) fn validate_stroke_line_value(
    field: &str,
    value: &str,
) -> Result<()> {
    match value {
        "butt" | "round" | "square" | "arcs" | "bevel" | "miter" | "miter-clip" => Ok(()),
        _ => Err(format!("Invalid {field} value (value: {value})").into()),
    }
}
pub(in crate::commands::icons) fn ensure_provider(provider: &str) -> Result<()> {
    if provider == "lucide" {
        Ok(())
    } else {
        Err(format!("Unsupported icon provider `{provider}`. Supported providers: lucide.").into())
    }
}

/// The `IconRole` variant a role name generates, derived rather than looked up
/// so a CLI build can generate code for a `nive` it has never seen. The app's
/// compiler resolves the variant against the version the app depends on.
pub(in crate::commands::icons) fn role_variant(role: &str) -> Result<String> {
    validate_role_name(role)?;

    Ok(role
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            let first = chars.next().expect("validated role has no empty segment");
            first.to_ascii_uppercase().to_string() + chars.as_str()
        })
        .collect())
}

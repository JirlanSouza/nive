use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use super::lucide::display_path;
use super::manifest::role_variant;
use super::{
    IconGenerationTarget, IconPaths, IconsManifest, LucideProviderConfig, ProviderRef, Result,
};

pub(super) fn ensure_icon_module_files(
    paths: &IconPaths,
    target: IconGenerationTarget,
) -> Result<()> {
    fs::create_dir_all(paths.generated_rs.parent().unwrap())?;
    fs::create_dir_all(paths.generated_catalog.parent().unwrap())?;

    write_if_changed(&paths.generated_rs, generated_module_source().as_bytes())?;
    if target == IconGenerationTarget::App {
        ensure_icons_rs(paths)?;
    }

    Ok(())
}

pub(super) fn ensure_icons_rs(paths: &IconPaths) -> Result<()> {
    let required_mod = "mod generated;\n";
    let required_export = "pub use generated::catalog::APP_ICON_CATALOG;\n#[allow(unused_imports)]\npub use generated::symbols::IconSymbol;\n";

    if !paths.icons_rs.exists() {
        write_if_changed(
            &paths.icons_rs,
            format!("{required_mod}\n{required_export}").as_bytes(),
        )?;
        return Ok(());
    }

    let mut contents = fs::read_to_string(&paths.icons_rs)?;
    let mut changed = false;

    if !contents.contains("mod generated;") {
        if !contents.ends_with('\n') {
            contents.push('\n');
        }
        contents.push('\n');
        contents.push_str(required_mod);
        changed = true;
    }

    if !contents.contains("APP_ICON_CATALOG") || !contents.contains("IconSymbol") {
        if !contents.ends_with('\n') {
            contents.push('\n');
        }
        contents.push_str(required_export);
        changed = true;
    }

    if changed {
        write_if_changed(&paths.icons_rs, contents.as_bytes())?;
    }

    Ok(())
}

pub(super) fn check_framework_generated_integration(paths: &IconPaths, failures: &mut Vec<String>) {
    let Ok(contents) = fs::read_to_string(&paths.icons_rs) else {
        failures.push(format!(
            "{} is missing or unreadable.",
            display_path(&paths.icons_rs)
        ));
        return;
    };

    if !contents.contains("mod generated;") && !contents.contains("pub mod generated;") {
        failures.push(format!(
            "{} does not compile the generated icon module.",
            display_path(&paths.icons_rs)
        ));
    }

    if !contents.contains("generated::catalog::APP_ICON_CATALOG") {
        failures.push(format!(
            "{} does not use the generated framework icon catalog.",
            display_path(&paths.icons_rs)
        ));
    }
}

pub(super) fn generated_module_source() -> &'static str {
    "pub mod catalog;\npub mod symbols;\n"
}

pub(super) fn normalize_svg(source: &str, config: &LucideProviderConfig) -> Result<String> {
    let start = source.find("<svg").ok_or("SVG root not found")?;
    let root_end = source[start..]
        .find('>')
        .map(|index| start + index)
        .ok_or("SVG root is not closed")?;
    let close = source.rfind("</svg>").ok_or("SVG closing tag not found")?;

    if close <= root_end {
        return Err("SVG closing tag appears before root content".into());
    }

    let inner = source[root_end + 1..close].trim();
    let stroke_width = config.stroke_width.as_str();
    let stroke_linecap = config.stroke_linecap.as_str();
    let stroke_linejoin = config.stroke_linejoin.as_str();
    let mut normalized = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="{stroke_width}" stroke-linecap="{stroke_linecap}" stroke-linejoin="{stroke_linejoin}">"#
    );
    normalized.push('\n');

    for line in inner.lines().map(str::trim).filter(|line| !line.is_empty()) {
        normalized.push_str("  ");
        normalized.push_str(line);
        normalized.push('\n');
    }

    normalized.push_str("</svg>\n");
    Ok(normalized)
}

pub(super) fn write_generated_modules(
    paths: &IconPaths,
    manifest: &IconsManifest,
    target: IconGenerationTarget,
) -> Result<()> {
    ensure_icon_module_files(paths, target)?;
    write_if_changed(
        &paths.generated_catalog,
        generate_catalog_source(manifest, target)?.as_bytes(),
    )?;
    write_if_changed(
        &paths.generated_symbols,
        generate_symbols_source(manifest, target)?.as_bytes(),
    )?;
    Ok(())
}

pub(super) fn generate_catalog_source(
    manifest: &IconsManifest,
    target: IconGenerationTarget,
) -> Result<String> {
    let import = match target {
        IconGenerationTarget::App => {
            "use nive::prelude::{IconCatalog, IconCatalogEntry, IconGlyph, IconRole};"
        }
        IconGenerationTarget::Framework => {
            "use crate::icons::{IconCatalog, IconCatalogEntry, IconGlyph, IconRole};"
        }
    };
    let mut source = format!(
        "// Bundled Lucide icons are sourced from Lucide (https://lucide.dev) and distributed under the ISC License.\n{import}\n\npub const APP_ICON_CATALOG: IconCatalog = IconCatalog::new(&[\n",
    );

    for (role_name, value) in &manifest.roles {
        let icon_ref = ProviderRef::parse(value)?;
        source.push_str("    IconCatalogEntry::new(\n");
        source.push_str("        IconRole::");
        source.push_str(role_variant(role_name)?);
        source.push_str(",\n");
        source.push_str("        IconGlyph::new(\n");
        source.push_str("            include_bytes!(\"");
        source.push_str(&include_path_for_ref(&icon_ref));
        source.push_str("\"),\n");
        source.push_str("            \"");
        source.push_str(&icon_ref.provider_slug());
        source.push_str("\",\n");
        source.push_str("        ),\n");
        source.push_str("    ),\n");
    }

    source.push_str("]);\n");
    Ok(source)
}

pub(super) fn generate_symbols_source(
    manifest: &IconsManifest,
    target: IconGenerationTarget,
) -> Result<String> {
    let import = match target {
        IconGenerationTarget::App => "use nive::prelude::{IconRef, IconSource};",
        IconGenerationTarget::Framework => "use crate::icons::{IconRef, IconSource};",
    };
    let mut source = format!(
        "// Bundled Lucide icons are sourced from Lucide (https://lucide.dev) and distributed under the ISC License.\n{import}\n\n#[allow(dead_code)]\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum IconSymbol",
    );

    if manifest.symbols.is_empty() {
        source.push_str(" {}\n\n");
    } else {
        source.push_str(" {\n");
        for variant in manifest.symbols.keys() {
            source.push_str("    ");
            source.push_str(variant);
            source.push_str(",\n");
        }
        source.push_str("}\n\n");
    }

    source.push_str("impl IconSource for IconSymbol {\n");
    source.push_str("    fn svg_bytes(self) -> &'static [u8] {\n");
    if manifest.symbols.is_empty() {
        source.push_str("        match self {}\n");
    } else {
        source.push_str("        match self {\n");

        for (variant, value) in &manifest.symbols {
            let icon_ref = ProviderRef::parse(value)?;
            source.push_str("            Self::");
            source.push_str(variant);
            source.push_str(" => include_bytes!(\"");
            source.push_str(&include_path_for_ref(&icon_ref));
            source.push_str("\"),\n");
        }

        source.push_str("        }\n");
    }

    source.push_str("    }\n\n");
    source.push_str("    fn provider_slug(self) -> &'static str {\n");
    if manifest.symbols.is_empty() {
        source.push_str("        match self {}\n");
    } else {
        source.push_str("        match self {\n");

        for (variant, value) in &manifest.symbols {
            let icon_ref = ProviderRef::parse(value)?;
            source.push_str("            Self::");
            source.push_str(variant);
            source.push_str(" => \"");
            source.push_str(&icon_ref.provider_slug());
            source.push_str("\",\n");
        }

        source.push_str("        }\n");
    }

    source.push_str("    }\n");
    source.push_str("}\n\n");
    // Lets a generated symbol drop straight into any widget icon slot.
    source.push_str("impl From<IconSymbol> for IconRef {\n");
    source.push_str("    fn from(symbol: IconSymbol) -> Self {\n");
    source.push_str("        Self::from_source(symbol)\n");
    source.push_str("    }\n");
    source.push_str("}\n");
    Ok(source)
}

pub(super) fn include_path_for_ref(icon_ref: &ProviderRef) -> String {
    let path =
        PathBuf::from("../../../assets/icons/generated").join(icon_ref.generated_asset_path());
    path.to_string_lossy().replace('\\', "/")
}

pub(super) fn collect_manifest_refs(manifest: &IconsManifest) -> Result<BTreeSet<ProviderRef>> {
    let mut refs = BTreeSet::new();

    for value in manifest.roles.values().chain(manifest.symbols.values()) {
        refs.insert(ProviderRef::parse(value)?);
    }

    Ok(refs)
}

pub(super) fn remove_stale_assets(paths: &IconPaths, manifest: &IconsManifest) -> Result<()> {
    let mut failures = Vec::new();
    collect_stale_assets(paths, manifest, &mut failures)?;

    for failure in failures {
        let Some(path) = failure.strip_suffix(" is not declared in icons.toml.") else {
            continue;
        };
        fs::remove_file(path)?;
        println!("Removed {path}");
    }

    Ok(())
}

pub(super) fn collect_stale_assets(
    paths: &IconPaths,
    manifest: &IconsManifest,
    failures: &mut Vec<String>,
) -> Result<()> {
    let expected_assets: BTreeSet<PathBuf> = collect_manifest_refs(manifest)?
        .into_iter()
        .map(|icon_ref| {
            paths
                .generated_asset_dir
                .join(icon_ref.generated_asset_path())
        })
        .collect();

    if !paths.generated_asset_dir.exists() {
        return Ok(());
    }

    for entry in walk_svg_files(&paths.generated_asset_dir)? {
        if !expected_assets.contains(&entry) {
            failures.push(format!(
                "{} is not declared in icons.toml.",
                display_path(&entry)
            ));
        }
    }

    Ok(())
}

pub(super) fn walk_svg_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !root.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walk_svg_files(&path)?);
        } else if path.extension() == Some(OsStr::new("svg")) {
            files.push(path);
        }
    }

    Ok(files)
}

pub(super) fn check_generated_file(failures: &mut Vec<String>, path: &Path, expected: &[u8]) {
    match fs::read(path) {
        Ok(actual) if actual == expected => {}
        Ok(_) => failures.push(format!(
            "{} is stale. Run `nive icons sync`.",
            display_path(path)
        )),
        Err(error) => failures.push(format!(
            "{} is missing or unreadable (error: {error})",
            display_path(path)
        )),
    }
}

pub(super) fn write_if_changed(path: &Path, contents: &[u8]) -> Result<bool> {
    if matches!(fs::read(path), Ok(existing) if existing == contents) {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;
    println!("Updated {}", display_path(path));
    Ok(true)
}

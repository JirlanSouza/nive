use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use clap::Subcommand;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct IconsManifest {
    version: String,
    stroke_width: String,
    stroke_linecap: String,
    stroke_linejoin: String,
    icons: BTreeMap<String, String>,
}

struct IconPaths {
    manifest: PathBuf,
    asset_dir: PathBuf,
    generated: PathBuf,
    widgets_mod: PathBuf,
    wrapper: PathBuf,
}

impl IconPaths {
    fn from_root(root: &Path) -> Self {
        Self {
            manifest: root.join("icons/lucide.toml"),
            asset_dir: root.join("assets/icons/lucide"),
            generated: root.join("src/widgets/icon.generated.rs"),
            widgets_mod: root.join("src/widgets/mod.rs"),
            wrapper: root.join("src/widgets/icon.rs"),
        }
    }
}

#[derive(Subcommand)]
pub enum IconsCommands {
    /// List icons declared in the current directory's manifest
    List,
    /// Fetch declared SVGs from Lucide and regenerate the icon Rust module
    Sync,
    /// Check if manifest, assets, and generated module are in sync
    Check,
    /// Add a new icon entry to the manifest and sync
    Add {
        /// PascalCase variant name (e.g., Shield)
        variant: String,
        /// Kebab-case Lucide icon name (e.g., shield-check)
        lucide_name: String,
    },
    /// Scaffold icons directory structure in the current directory
    Init,
}

pub fn run(command: IconsCommands) -> Result<()> {
    let cwd = std::env::current_dir()?;
    run_in_dir(command, &cwd)
}

fn run_in_dir(command: IconsCommands, root: &Path) -> Result<()> {
    let paths = IconPaths::from_root(root);

    match command {
        IconsCommands::List => icons_list(&paths),
        IconsCommands::Sync => icons_sync(&paths),
        IconsCommands::Check => icons_check(&paths),
        IconsCommands::Add {
            variant,
            lucide_name,
        } => icons_add(&paths, &variant, &lucide_name),
        IconsCommands::Init => icons_init(&paths),
    }
}

fn icons_list(paths: &IconPaths) -> Result<()> {
    require_manifest(paths)?;

    let manifest = read_manifest(&paths.manifest)?;

    for (variant, slug) in manifest.icons {
        println!("{variant} -> {slug}");
    }

    Ok(())
}

fn icons_sync(paths: &IconPaths) -> Result<()> {
    require_manifest(paths)?;
    ensure_icon_module_files(paths)?;

    let manifest = read_manifest(&paths.manifest)?;

    fs::create_dir_all(&paths.asset_dir)?;

    for slug in manifest.icons.values() {
        let source = fetch_lucide_svg(&manifest.version, slug)?;
        let normalized = normalize_svg(&source, &manifest)?;
        write_if_changed(
            &paths.asset_dir.join(format!("{slug}.svg")),
            normalized.as_bytes(),
        )?;
    }

    remove_stale_assets(&paths.asset_dir, &manifest)?;

    let generated = generate_icon_source(&manifest);
    write_if_changed(&paths.generated, generated.as_bytes())?;

    Ok(())
}

fn icons_check(paths: &IconPaths) -> Result<()> {
    require_manifest(paths)?;

    let manifest = read_manifest(&paths.manifest)?;
    let mut failures = Vec::new();

    let expected_generated = generate_icon_source(&manifest);
    match fs::read_to_string(&paths.generated) {
        Ok(actual) if actual == expected_generated => {}
        Ok(_) => failures.push(format!(
            "{} is stale. Run `nive icons sync`.",
            display_path(&paths.generated)
        )),
        Err(error) => failures.push(format!(
            "{} is missing or unreadable (error: {error})",
            display_path(&paths.generated)
        )),
    }

    for slug in manifest.icons.values() {
        let path = paths.asset_dir.join(format!("{slug}.svg"));
        match fs::read_to_string(&path) {
            Ok(actual) => {
                let expected = normalize_svg(&actual, &manifest)?;
                if actual != expected {
                    failures.push(format!(
                        "{} is not normalized. Run `nive icons sync`.",
                        display_path(&path)
                    ));
                }
            }
            Err(error) => failures.push(format!(
                "{} is missing or unreadable (error: {error})",
                display_path(&path)
            )),
        }
    }

    if paths.asset_dir.exists() {
        let expected_assets: BTreeSet<String> = manifest
            .icons
            .values()
            .map(|slug| format!("{slug}.svg"))
            .collect();

        for entry in fs::read_dir(&paths.asset_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension() != Some(OsStr::new("svg")) {
                continue;
            }

            let Some(file_name) = path.file_name().and_then(OsStr::to_str) else {
                continue;
            };

            if !expected_assets.contains(file_name) {
                failures.push(format!(
                    "{} is not declared in {}.",
                    display_path(&path),
                    display_path(&paths.manifest)
                ));
            }
        }
    }

    if failures.is_empty() {
        println!("Icon assets are up to date.");
        return Ok(());
    }

    for failure in &failures {
        eprintln!("{failure}");
    }

    Err(format!("Icon check failed: {} issue(s)", failures.len()).into())
}

fn icons_add(paths: &IconPaths, variant: &str, lucide_name: &str) -> Result<()> {
    validate_variant(variant)?;
    validate_slug(lucide_name)?;

    require_manifest(paths)?;

    let mut manifest = read_manifest(&paths.manifest)?;
    manifest
        .icons
        .insert(variant.to_string(), lucide_name.to_string());

    let updated = toml::to_string_pretty(&manifest)?;
    write_if_changed(&paths.manifest, updated.as_bytes())?;

    icons_sync(paths)
}

fn icons_init(paths: &IconPaths) -> Result<()> {
    fs::create_dir_all(paths.manifest.parent().unwrap())?;
    fs::create_dir_all(&paths.asset_dir)?;
    fs::create_dir_all(paths.generated.parent().unwrap())?;

    if paths.manifest.exists() {
        println!("Already exists {}", paths.manifest.display());
    } else {
        let empty_manifest = empty_manifest();
        let toml_string = toml::to_string_pretty(&empty_manifest)?;
        fs::write(&paths.manifest, toml_string)?;
        println!("Created {}", paths.manifest.display());
    }

    println!("Created {}", paths.asset_dir.display());
    ensure_icon_module_files(paths)?;

    if !paths.generated.exists() {
        let manifest = read_manifest(&paths.manifest)?;
        fs::write(&paths.generated, generate_icon_source(&manifest))?;
        println!("Created {}", paths.generated.display());
    }

    println!("\nNext steps:");
    println!("  nive icons add <Variant> <lucide-name>");
    println!("  nive icons sync");

    Ok(())
}

fn empty_manifest() -> IconsManifest {
    IconsManifest {
        version: "0.460.0".to_string(),
        stroke_width: "2".to_string(),
        stroke_linecap: "round".to_string(),
        stroke_linejoin: "round".to_string(),
        icons: BTreeMap::new(),
    }
}

fn require_manifest(paths: &IconPaths) -> Result<()> {
    if paths.manifest.exists() {
        return Ok(());
    }

    Err(format!(
        "No `icons/lucide.toml` found at {}. Run `nive icons init` first.",
        paths.manifest.display()
    )
    .into())
}

fn ensure_icon_module_files(paths: &IconPaths) -> Result<()> {
    fs::create_dir_all(paths.generated.parent().unwrap())?;

    if paths.widgets_mod.exists() {
        let contents = fs::read_to_string(&paths.widgets_mod)?;
        if !contents.contains("pub mod icon;") {
            println!(
                "{} already exists; add `pub mod icon;` to expose generated icons.",
                paths.widgets_mod.display()
            );
        }
    } else {
        fs::write(&paths.widgets_mod, "pub mod icon;\n")?;
        println!("Created {}", paths.widgets_mod.display());
    }

    if paths.wrapper.exists() {
        println!("Already exists {}", paths.wrapper.display());
    } else {
        fs::write(&paths.wrapper, icon_wrapper_source())?;
        println!("Created {}", paths.wrapper.display());
    }

    Ok(())
}

fn icon_wrapper_source() -> &'static str {
    "use nive::prelude::IconSource;\n\ninclude!(\"icon.generated.rs\");\n"
}

fn read_manifest(path: &Path) -> Result<IconsManifest> {
    let contents = fs::read_to_string(path)?;
    let manifest: IconsManifest = toml::from_str(&contents)?;

    if manifest.version.trim().is_empty() {
        return Err("Lucide version cannot be empty".into());
    }

    if manifest.stroke_width.trim().is_empty() {
        return Err("Lucide stroke_width cannot be empty".into());
    }

    validate_stroke_line_value("stroke_linecap", &manifest.stroke_linecap)?;
    validate_stroke_line_value("stroke_linejoin", &manifest.stroke_linejoin)?;

    for (variant, slug) in &manifest.icons {
        validate_variant(variant)?;
        validate_slug(slug)?;
    }

    Ok(manifest)
}

fn fetch_lucide_svg(version: &str, slug: &str) -> Result<String> {
    let url =
        format!("https://raw.githubusercontent.com/lucide-icons/lucide/{version}/icons/{slug}.svg");

    let mut response = ureq::get(&url)
        .call()
        .map_err(|error| format!("Failed to fetch {url}: {error}"))?;

    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|error| format!("Failed to read response for {url}: {error}"))?;

    Ok(body)
}

fn normalize_svg(source: &str, manifest: &IconsManifest) -> Result<String> {
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
    let stroke_width = manifest.stroke_width.as_str();
    let stroke_linecap = manifest.stroke_linecap.as_str();
    let stroke_linejoin = manifest.stroke_linejoin.as_str();
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

fn generate_icon_source(manifest: &IconsManifest) -> String {
    let mut source = String::from(
        "// Bundled icons are sourced from Lucide (https://lucide.dev) and distributed under the ISC License.\n\
         #[allow(dead_code)]\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub enum IconName {\n",
    );

    for variant in manifest.icons.keys() {
        source.push_str("    ");
        source.push_str(variant);
        source.push_str(",\n");
    }

    source.push_str("}\n\nimpl IconSource for IconName {\n");
    source.push_str("    fn svg_bytes(self) -> &'static [u8] {\n");
    source.push_str("        match self {\n");

    for (variant, slug) in &manifest.icons {
        source.push_str("            Self::");
        source.push_str(variant);
        source.push_str(" => include_bytes!(\"");
        source.push_str("../../assets/icons/lucide/");
        source.push_str(slug);
        source.push_str(".svg\"),\n");
    }

    source.push_str("        }\n");
    source.push_str("    }\n\n");
    source.push_str("    fn provider_slug(self) -> &'static str {\n");
    source.push_str("        match self {\n");

    for (variant, slug) in &manifest.icons {
        source.push_str("            Self::");
        source.push_str(variant);
        source.push_str(" => \"");
        source.push_str(slug);
        source.push_str("\",\n");
    }

    source.push_str("        }\n");
    source.push_str("    }\n");
    source.push_str("}\n");
    source
}

fn write_if_changed(path: &Path, contents: &[u8]) -> Result<bool> {
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

fn remove_stale_assets(asset_dir: &Path, manifest: &IconsManifest) -> Result<()> {
    let expected_assets: BTreeSet<String> = manifest
        .icons
        .values()
        .map(|slug| format!("{slug}.svg"))
        .collect();

    for entry in fs::read_dir(asset_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension() != Some(OsStr::new("svg")) {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };

        if !expected_assets.contains(file_name) {
            fs::remove_file(&path)?;
            println!("Removed {}", display_path(&path));
        }
    }

    Ok(())
}

fn validate_variant(variant: &str) -> Result<()> {
    let mut chars = variant.chars();
    let Some(first) = chars.next() else {
        return Err("App icon variant cannot be empty".into());
    };

    if !first.is_ascii_uppercase() {
        return Err(format!(
            "App icon variant must start with uppercase ASCII (variant: {variant})"
        )
        .into());
    }

    if !chars.all(|ch| ch.is_ascii_alphanumeric()) {
        return Err(
            format!("App icon variant must be PascalCase ASCII (variant: {variant})").into(),
        );
    }

    Ok(())
}

fn validate_slug(slug: &str) -> Result<()> {
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

fn validate_stroke_line_value(field: &str, value: &str) -> Result<()> {
    match value {
        "butt" | "round" | "square" | "arcs" | "bevel" | "miter" | "miter-clip" => Ok(()),
        _ => Err(format!("Invalid {field} value (value: {value})").into()),
    }
}

fn display_path(path: &Path) -> String {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.strip_prefix(&cwd)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest_with_icons(icons: &[(&str, &str)]) -> IconsManifest {
        let mut manifest = empty_manifest();
        manifest.icons = icons
            .iter()
            .map(|(variant, slug)| ((*variant).to_string(), (*slug).to_string()))
            .collect();
        manifest
    }

    fn write_manifest(path: &Path, manifest: &IconsManifest) {
        fs::create_dir_all(path.parent().unwrap()).expect("create manifest parent");
        fs::write(
            path,
            toml::to_string_pretty(manifest).expect("serialize manifest"),
        )
        .expect("write manifest");
    }

    #[test]
    fn path_planning_uses_standard_app_icon_locations() {
        let root = Path::new("/tmp/demo-app");
        let paths = IconPaths::from_root(root);

        assert_eq!(paths.manifest, root.join("icons/lucide.toml"));
        assert_eq!(paths.asset_dir, root.join("assets/icons/lucide"));
        assert_eq!(paths.generated, root.join("src/widgets/icon.generated.rs"));
        assert_eq!(paths.widgets_mod, root.join("src/widgets/mod.rs"));
        assert_eq!(paths.wrapper, root.join("src/widgets/icon.rs"));
    }

    #[test]
    fn manifest_validation_rejects_invalid_icon_slug() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let paths = IconPaths::from_root(tempdir.path());
        write_manifest(
            &paths.manifest,
            &manifest_with_icons(&[("BadSlug", "bad_slug")]),
        );

        let error = read_manifest(&paths.manifest).expect_err("invalid slug should fail");

        assert!(
            error.to_string().contains("kebab-case ASCII"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn source_generation_implements_icon_source_contract() {
        let manifest = manifest_with_icons(&[("Shield", "shield-check")]);

        let source = generate_icon_source(&manifest);

        assert!(source.contains("pub enum IconName"));
        assert!(source.contains("impl IconSource for IconName"));
        assert!(source.contains(
            "Self::Shield => include_bytes!(\"../../assets/icons/lucide/shield-check.svg\")"
        ));
        assert!(source.contains("Self::Shield => \"shield-check\""));
    }

    #[test]
    fn generated_module_integration_files_are_created_without_network() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let paths = IconPaths::from_root(tempdir.path());

        icons_init(&paths).expect("init icons");

        assert_eq!(
            fs::read_to_string(&paths.widgets_mod).expect("read widgets mod"),
            "pub mod icon;\n"
        );
        assert_eq!(
            fs::read_to_string(&paths.wrapper).expect("read wrapper"),
            icon_wrapper_source()
        );
        assert!(fs::read_to_string(&paths.generated)
            .expect("read generated")
            .contains("impl IconSource for IconName"));
    }

    #[test]
    fn init_does_not_overwrite_user_authored_wrapper_files() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let paths = IconPaths::from_root(tempdir.path());
        fs::create_dir_all(paths.wrapper.parent().unwrap()).expect("create widgets dir");
        fs::write(&paths.widgets_mod, "// custom widgets\n").expect("write custom mod");
        fs::write(&paths.wrapper, "// custom icon wrapper\n").expect("write custom wrapper");

        icons_init(&paths).expect("init icons");

        assert_eq!(
            fs::read_to_string(&paths.widgets_mod).expect("read custom mod"),
            "// custom widgets\n"
        );
        assert_eq!(
            fs::read_to_string(&paths.wrapper).expect("read custom wrapper"),
            "// custom icon wrapper\n"
        );
    }

    #[test]
    fn stale_assets_are_removed_offline() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let paths = IconPaths::from_root(tempdir.path());
        fs::create_dir_all(&paths.asset_dir).expect("create asset dir");
        fs::write(paths.asset_dir.join("shield-check.svg"), "<svg></svg>").expect("write icon");
        fs::write(paths.asset_dir.join("stale.svg"), "<svg></svg>").expect("write stale icon");

        remove_stale_assets(
            &paths.asset_dir,
            &manifest_with_icons(&[("Shield", "shield-check")]),
        )
        .expect("remove stale assets");

        assert!(paths.asset_dir.join("shield-check.svg").exists());
        assert!(!paths.asset_dir.join("stale.svg").exists());
    }

    #[test]
    fn command_check_returns_missing_manifest_error() {
        let tempdir = tempfile::tempdir().expect("tempdir");

        let error =
            run_in_dir(IconsCommands::Check, tempdir.path()).expect_err("missing manifest fails");

        assert!(
            error.to_string().contains("No `icons/lucide.toml` found"),
            "unexpected error: {error}"
        );
    }
}

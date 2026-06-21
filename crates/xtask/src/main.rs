use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const MANIFEST_PATH: &str = "crates/nive-ui/icons/lucide.toml";
const ASSET_DIR: &str = "crates/nive-ui/assets/icons/lucide";
const GENERATED_PATH: &str = "crates/nive-ui/src/widgets/icon.generated.rs";

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
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    match (args.next().as_deref(), args.next().as_deref()) {
        (Some("icons"), Some("list")) => icons_list(),
        (Some("icons"), Some("sync")) => icons_sync(),
        (Some("icons"), Some("check")) => icons_check(),
        (Some("icons"), Some("add")) => {
            let variant = args.next().ok_or("Missing app icon variant")?;
            let lucide_name = args.next().ok_or("Missing Lucide icon name")?;
            if args.next().is_some() {
                return Err("icons add accepts exactly two arguments".into());
            }
            icons_add(&variant, &lucide_name)
        }
        (Some("icons"), Some("init")) => {
            let app_path = args.next().ok_or("Missing app crate path")?;
            icons_init(&app_path)
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  cargo run --package xtask -- icons list");
            eprintln!("  cargo run --package xtask -- icons sync");
            eprintln!("  cargo run --package xtask -- icons check");
            eprintln!("  cargo run --package xtask -- icons add <Variant> <lucide-name>");
            eprintln!("  cargo run --package xtask -- icons init <app-crate-path>");
            Err("Unknown xtask command".into())
        }
    }
}

fn icons_list() -> Result<()> {
    let manifest = read_manifest(&paths()?.manifest)?;

    for (variant, slug) in manifest.icons {
        println!("{variant} -> {slug}");
    }

    Ok(())
}

fn icons_sync() -> Result<()> {
    let paths = paths()?;
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

fn icons_check() -> Result<()> {
    let paths = paths()?;
    let manifest = read_manifest(&paths.manifest)?;
    let mut failures = Vec::new();

    let expected_generated = generate_icon_source(&manifest);
    match fs::read_to_string(&paths.generated) {
        Ok(actual) if actual == expected_generated => {}
        Ok(_) => failures.push(format!(
            "{} is stale. Run `just icons-sync`.",
            display_path(&paths.generated)
        )),
        Err(error) => failures.push(format!(
            "{} is missing or unreadable (error: {error})",
            display_path(&paths.generated)
        )),
    }

    let expected_assets: BTreeSet<String> = manifest
        .icons
        .values()
        .map(|slug| format!("{slug}.svg"))
        .collect();

    for slug in manifest.icons.values() {
        let path = paths.asset_dir.join(format!("{slug}.svg"));
        match fs::read_to_string(&path) {
            Ok(actual) => {
                let expected = normalize_svg(&actual, &manifest)?;
                if actual != expected {
                    failures.push(format!(
                        "{} is not normalized. Run `just icons-sync`.",
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

    for failure in failures {
        eprintln!("{failure}");
    }

    Err("Icon check failed".into())
}

fn icons_add(variant: &str, lucide_name: &str) -> Result<()> {
    validate_variant(variant)?;
    validate_slug(lucide_name)?;

    let paths = paths()?;
    let mut manifest = read_manifest(&paths.manifest)?;
    manifest
        .icons
        .insert(variant.to_string(), lucide_name.to_string());

    let updated = toml::to_string_pretty(&manifest)?;
    write_if_changed(&paths.manifest, updated.as_bytes())?;

    icons_sync()
}

fn icons_init(app_path: &str) -> Result<()> {
    let app_root = PathBuf::from(app_path);

    if !app_root.exists() {
        return Err(format!("App path does not exist: {}", app_path).into());
    }

    let manifest_path = app_root.join("icons/lucide.toml");
    let asset_dir = app_root.join("assets/icons/lucide");
    let generated_path = app_root.join("src/widgets/icon.generated.rs");

    if manifest_path.exists() {
        return Err(format!("Manifest already exists: {}", manifest_path.display()).into());
    }

    fs::create_dir_all(manifest_path.parent().unwrap())?;
    fs::create_dir_all(&asset_dir)?;
    fs::create_dir_all(generated_path.parent().unwrap())?;

    let empty_manifest = IconsManifest {
        version: "0.460.0".to_string(),
        stroke_width: "2".to_string(),
        stroke_linecap: "round".to_string(),
        stroke_linejoin: "round".to_string(),
        icons: BTreeMap::new(),
    };

    let toml_string = toml::to_string_pretty(&empty_manifest)?;
    fs::write(&manifest_path, toml_string)?;

    println!("Created {}", manifest_path.display());
    println!("Created {}", asset_dir.display());
    println!("Created {}", generated_path.parent().unwrap().display());
    println!("\nNext steps:");
    println!("  just icons-add <Variant> <lucide-name>");
    println!("  just icons-sync");

    Ok(())
}

fn paths() -> Result<IconPaths> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("Could not resolve repository root")?
        .to_path_buf();

    Ok(IconPaths {
        manifest: root.join(MANIFEST_PATH),
        asset_dir: root.join(ASSET_DIR),
        generated: root.join(GENERATED_PATH),
    })
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
    let output = Command::new("curl")
        .args(["--fail", "--location", "--silent", "--show-error", &url])
        .output()
        .map_err(|error| format!("Failed to run curl for {url} (error: {error})"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to fetch {url} ({stderr})").into());
    }

    String::from_utf8(output.stdout).map_err(|error| {
        format!("Lucide SVG response was not UTF-8 (icon: {slug}, error: {error})").into()
    })
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
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub enum Icon {\n",
    );

    for variant in manifest.icons.keys() {
        source.push_str("    ");
        source.push_str(variant);
        source.push_str(",\n");
    }

    source.push_str("}\n\nimpl Icon {\n");
    source.push_str("    fn svg_bytes(&self) -> &'static [u8] {\n");
    source.push_str("        match self {\n");

    for (variant, slug) in &manifest.icons {
        source.push_str("            Self::");
        source.push_str(variant);
        source.push_str(" => include_bytes!(\"");
        source.push_str("../../../assets/icons/lucide/");
        source.push_str(slug);
        source.push_str(".svg\"),\n");
    }

    source.push_str("        }\n");
    source.push_str("    }\n\n");
    source.push_str("    #[allow(dead_code)]\n");
    source.push_str("    pub(crate) fn provider_slug(&self) -> &'static str {\n");
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
    path.strip_prefix(paths_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn paths_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

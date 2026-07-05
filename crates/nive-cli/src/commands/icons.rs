use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    process::Command,
};

use clap::Subcommand;
use flate2::read::GzDecoder;
use serde::{Deserialize, Deserializer, Serialize};
use tar::Archive;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct IconsManifest {
    #[serde(default)]
    provider: ProviderConfig,
    #[serde(default)]
    roles: BTreeMap<String, String>,
    #[serde(default)]
    symbols: BTreeMap<String, String>,
    #[serde(default)]
    custom: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProviderConfig {
    #[serde(default)]
    lucide: LucideProviderConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct LucideProviderConfig {
    version: String,
    stroke_width: String,
    stroke_linecap: String,
    stroke_linejoin: String,
}

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ProviderRef {
    Lucide(String),
    Custom(String),
}

impl ProviderRef {
    fn parse(value: &str) -> Result<Self> {
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

    fn parse_command_input(value: &str) -> Result<Self> {
        if value.contains(':') {
            return Self::parse(value);
        }

        validate_lucide_slug(value)?;
        Ok(Self::Lucide(value.to_string()))
    }

    fn normalized(&self) -> String {
        match self {
            Self::Lucide(slug) => format!("lucide:{slug}"),
            Self::Custom(name) => format!("custom:{name}"),
        }
    }

    fn provider_slug(&self) -> String {
        self.normalized()
    }

    fn generated_asset_path(&self) -> PathBuf {
        match self {
            Self::Lucide(slug) => PathBuf::from("lucide").join(format!("{slug}.svg")),
            Self::Custom(name) => PathBuf::from("custom").join(format!("{name}.svg")),
        }
    }
}

struct IconPaths {
    root: PathBuf,
    manifest: PathBuf,
    generated_asset_dir: PathBuf,
    icons_rs: PathBuf,
    generated_rs: PathBuf,
    generated_catalog: PathBuf,
    generated_symbols: PathBuf,
    gallery_dir: PathBuf,
    metadata_cache_dir: PathBuf,
    legacy_manifest: PathBuf,
}

impl IconPaths {
    fn from_root(root: &Path) -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IconGenerationTarget {
    App,
    Framework,
}

#[derive(Subcommand)]
pub enum IconsCommands {
    /// List icons declared in the current manifest, or list provider metadata with --provider
    List {
        /// Provider to list from discovery metadata
        #[arg(long)]
        provider: Option<String>,
        /// Provider category to filter
        #[arg(long)]
        category: Option<String>,
    },
    /// Fetch declared SVGs and regenerate named icon Rust modules
    Sync {
        /// Generate framework-internal modules for the nive-ui crate
        #[arg(long, default_value_t = false)]
        framework: bool,
    },
    /// Check if manifest, generated assets, and generated modules are in sync without network
    Check {
        /// Check framework-internal modules for the nive-ui crate
        #[arg(long, default_value_t = false)]
        framework: bool,
    },
    /// Add an app-owned symbol to icons.toml and sync
    #[command(name = "add-symbol")]
    AddSymbol {
        /// PascalCase symbol variant (e.g., User)
        variant: String,
        /// Provider ref, e.g. lucide:user or custom:brand-mark. Bare values are Lucide shorthand.
        provider_ref: String,
    },
    /// Map a semantic role to a provider ref and sync
    #[command(name = "set-role")]
    SetRole {
        /// Canonical role name, e.g. window-close
        role_name: String,
        /// Provider ref, e.g. lucide:x or custom:brand-close. Bare values are Lucide shorthand.
        provider_ref: String,
    },
    /// Register a custom SVG in icons.toml
    #[command(name = "add-custom")]
    AddCustom {
        /// Custom icon name used by custom:<name> refs
        name: String,
        /// Manifest-relative SVG path
        path: String,
    },
    /// Scaffold icons.toml and named generated modules in the current app
    Init,
    /// Search Lucide provider metadata
    Search {
        query: String,
        #[arg(long, default_value = "lucide")]
        provider: String,
    },
    /// Show provider metadata for one icon ref
    Show { provider_ref: String },
    /// Generate a local provider gallery HTML file
    Gallery {
        #[arg(long, default_value = "lucide")]
        provider: String,
        #[arg(long, default_value_t = false)]
        open: bool,
    },
    /// Deprecated. Use add-symbol instead.
    Add {
        variant: String,
        provider_ref: String,
    },
}

pub fn run(command: IconsCommands) -> Result<()> {
    let cwd = std::env::current_dir()?;
    run_in_dir(command, &cwd)
}

fn run_in_dir(command: IconsCommands, root: &Path) -> Result<()> {
    let paths = IconPaths::from_root(root);

    match command {
        IconsCommands::List { provider, category } => match provider {
            Some(provider) => icons_provider_list(&paths, &provider, category.as_deref()),
            None => {
                if category.is_some() {
                    return Err("`--category` requires `--provider lucide`.".into());
                }
                icons_list_manifest(&paths)
            }
        },
        IconsCommands::Sync { framework } => icons_sync(&paths, generation_target(framework)),
        IconsCommands::Check { framework } => icons_check(&paths, generation_target(framework)),
        IconsCommands::AddSymbol {
            variant,
            provider_ref,
        } => icons_add_symbol(&paths, &variant, &provider_ref),
        IconsCommands::SetRole {
            role_name,
            provider_ref,
        } => icons_set_role(&paths, &role_name, &provider_ref),
        IconsCommands::AddCustom { name, path } => icons_add_custom(&paths, &name, &path),
        IconsCommands::Init => icons_init(&paths),
        IconsCommands::Search { query, provider } => icons_search(&paths, &provider, &query),
        IconsCommands::Show { provider_ref } => icons_show(&paths, &provider_ref),
        IconsCommands::Gallery { provider, open } => icons_gallery(&paths, &provider, open),
        IconsCommands::Add { .. } => Err(
            "`nive icons add` has been replaced by `nive icons add-symbol <Variant> <provider-ref>`."
                .into(),
        ),
    }
}

fn generation_target(framework: bool) -> IconGenerationTarget {
    if framework {
        IconGenerationTarget::Framework
    } else {
        IconGenerationTarget::App
    }
}

fn icons_list_manifest(paths: &IconPaths) -> Result<()> {
    require_manifest(paths)?;
    let manifest = read_manifest(&paths.manifest)?;

    println!("[roles]");
    for (role, icon_ref) in &manifest.roles {
        println!("{role} -> {icon_ref}");
    }

    println!("\n[symbols]");
    for (symbol, icon_ref) in &manifest.symbols {
        println!("{symbol} -> {icon_ref}");
    }

    if !manifest.custom.is_empty() {
        println!("\n[custom]");
        for (name, path) in &manifest.custom {
            println!("{name} -> {path}");
        }
    }

    Ok(())
}

fn icons_sync(paths: &IconPaths, target: IconGenerationTarget) -> Result<()> {
    require_manifest(paths)?;
    ensure_icon_module_files(paths, target)?;

    let manifest = read_manifest(&paths.manifest)?;
    let refs = collect_manifest_refs(&manifest)?;

    fs::create_dir_all(&paths.generated_asset_dir)?;

    for icon_ref in refs {
        let source = match &icon_ref {
            ProviderRef::Lucide(slug) => fetch_lucide_svg(&manifest.provider.lucide.version, slug)?,
            ProviderRef::Custom(name) => read_custom_svg(paths, &manifest, name)?,
        };
        let normalized = normalize_svg(&source, &manifest.provider.lucide)?;
        write_if_changed(
            &paths
                .generated_asset_dir
                .join(icon_ref.generated_asset_path()),
            normalized.as_bytes(),
        )?;
    }

    remove_stale_assets(paths, &manifest)?;
    write_generated_modules(paths, &manifest, target)?;

    Ok(())
}

fn icons_check(paths: &IconPaths, target: IconGenerationTarget) -> Result<()> {
    require_manifest(paths)?;

    let manifest = read_manifest(&paths.manifest)?;
    let mut failures = Vec::new();

    for role in required_role_names() {
        if !manifest.roles.contains_key(role) {
            failures.push(format!(
                "{} is missing required icon role `{role}`.",
                display_path(&paths.manifest)
            ));
        }
    }

    for (name, source_path) in &manifest.custom {
        if let Err(error) = validate_custom_svg_path(paths, source_path) {
            failures.push(format!(
                "{} custom icon `{name}` is invalid: {error}",
                display_path(&paths.manifest)
            ));
        }
    }

    check_generated_file(
        &mut failures,
        &paths.generated_rs,
        generated_module_source().as_bytes(),
    );
    check_generated_file(
        &mut failures,
        &paths.generated_catalog,
        generate_catalog_source(&manifest, target)?.as_bytes(),
    );
    check_generated_file(
        &mut failures,
        &paths.generated_symbols,
        generate_symbols_source(&manifest, target)?.as_bytes(),
    );

    if target == IconGenerationTarget::Framework {
        check_framework_generated_integration(paths, &mut failures);
    }

    for icon_ref in collect_manifest_refs(&manifest)? {
        let path = paths
            .generated_asset_dir
            .join(icon_ref.generated_asset_path());
        match fs::read_to_string(&path) {
            Ok(actual) => {
                let expected = match &icon_ref {
                    ProviderRef::Lucide(_) => normalize_svg(&actual, &manifest.provider.lucide)?,
                    ProviderRef::Custom(name) => normalize_svg(
                        &read_custom_svg(paths, &manifest, name)?,
                        &manifest.provider.lucide,
                    )?,
                };

                if actual != expected {
                    failures.push(format!(
                        "{} is stale or not normalized. Run `nive icons sync`.",
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

    collect_stale_assets(paths, &manifest, &mut failures)?;

    if failures.is_empty() {
        println!("Icon assets and generated modules are up to date.");
        return Ok(());
    }

    for failure in &failures {
        eprintln!("{failure}");
    }

    Err(format!("Icon check failed: {} issue(s)", failures.len()).into())
}

fn icons_add_symbol(paths: &IconPaths, variant: &str, value: &str) -> Result<()> {
    validate_variant(variant)?;
    require_manifest(paths)?;

    let icon_ref = ProviderRef::parse_command_input(value)?;
    let mut manifest = read_manifest(&paths.manifest)?;
    validate_ref_custom_target(&manifest, &icon_ref)?;
    manifest
        .symbols
        .insert(variant.to_string(), icon_ref.normalized());

    write_manifest(paths, &manifest)?;
    icons_sync(paths, IconGenerationTarget::App)
}

fn icons_set_role(paths: &IconPaths, role_name: &str, value: &str) -> Result<()> {
    validate_role_name(role_name)?;
    require_manifest(paths)?;

    let icon_ref = ProviderRef::parse_command_input(value)?;
    let mut manifest = read_manifest(&paths.manifest)?;
    validate_ref_custom_target(&manifest, &icon_ref)?;
    manifest
        .roles
        .insert(role_name.to_string(), icon_ref.normalized());

    write_manifest(paths, &manifest)?;
    icons_sync(paths, IconGenerationTarget::App)
}

fn icons_add_custom(paths: &IconPaths, name: &str, source_path: &str) -> Result<()> {
    validate_ref_name(name)?;
    require_manifest(paths)?;
    validate_custom_svg_path(paths, source_path)?;

    let mut manifest = read_manifest(&paths.manifest)?;
    manifest
        .custom
        .insert(name.to_string(), source_path.to_string());

    write_manifest(paths, &manifest)?;
    Ok(())
}

fn icons_init(paths: &IconPaths) -> Result<()> {
    fs::create_dir_all(&paths.generated_asset_dir)?;
    ensure_icon_module_files(paths, IconGenerationTarget::App)?;

    if paths.manifest.exists() {
        println!("Already exists {}", paths.manifest.display());
    } else {
        write_manifest(paths, &empty_manifest())?;
        println!("Created {}", paths.manifest.display());
    }

    if !paths.generated_catalog.exists() || !paths.generated_symbols.exists() {
        let manifest = read_manifest(&paths.manifest)?;
        write_generated_modules(paths, &manifest, IconGenerationTarget::App)?;
    }

    println!("\nNext steps:");
    println!("  nive icons add-symbol User user");
    println!("  nive icons set-role window-close lucide:x");
    println!("  nive icons sync");

    Ok(())
}

fn icons_search(paths: &IconPaths, provider: &str, query: &str) -> Result<()> {
    ensure_provider(provider)?;
    let version = lucide_provider_version(paths)?;
    let metadata = LucideProvider::new(paths, &version).metadata()?;
    let query = query.to_ascii_lowercase();

    for icon in metadata.search(&query) {
        println!("{}", icon.summary_line());
    }

    Ok(())
}

fn icons_provider_list(paths: &IconPaths, provider: &str, category: Option<&str>) -> Result<()> {
    ensure_provider(provider)?;
    let version = lucide_provider_version(paths)?;
    let metadata = LucideProvider::new(paths, &version).metadata()?;

    for icon in metadata.list(category) {
        println!("{}", icon.summary_line());
    }

    Ok(())
}

fn icons_show(paths: &IconPaths, value: &str) -> Result<()> {
    let ProviderRef::Lucide(slug) = ProviderRef::parse(value)? else {
        return Err(
            "`nive icons show` currently supports Lucide refs such as `lucide:user`.".into(),
        );
    };

    let version = lucide_provider_version(paths)?;
    let metadata = LucideProvider::new(paths, &version).metadata()?;
    let Some(icon) = metadata.find(&slug) else {
        return Err(format!("Lucide metadata does not contain `lucide:{slug}`.").into());
    };

    println!("ref: lucide:{}", icon.name);
    println!("name: {}", icon.name);
    print_metadata_list("aliases", &icon.aliases);
    print_metadata_list("tags", &icon.tags);
    print_metadata_list("categories", &icon.categories);
    print_metadata_list("use-cases", &icon.use_cases);
    Ok(())
}

fn icons_gallery(paths: &IconPaths, provider: &str, open: bool) -> Result<()> {
    ensure_provider(provider)?;
    fs::create_dir_all(&paths.gallery_dir)?;

    let version = lucide_provider_version(paths)?;
    let metadata = LucideProvider::new(paths, &version).metadata()?;
    let gallery_path = paths.gallery_dir.join("lucide-gallery.html");
    let html = generate_gallery_html(&metadata);
    write_if_changed(&gallery_path, html.as_bytes())?;
    println!("{}", display_path(&gallery_path));

    if open {
        open_path(&gallery_path)?;
    }

    Ok(())
}

fn lucide_provider_version(paths: &IconPaths) -> Result<String> {
    if paths.manifest.exists() {
        return Ok(read_manifest(&paths.manifest)?.provider.lucide.version);
    }

    Ok(LucideProviderConfig::default().version)
}

fn empty_manifest() -> IconsManifest {
    IconsManifest {
        provider: ProviderConfig::default(),
        roles: default_role_refs(),
        symbols: BTreeMap::new(),
        custom: BTreeMap::new(),
    }
}

fn require_manifest(paths: &IconPaths) -> Result<()> {
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

fn ensure_icon_module_files(paths: &IconPaths, target: IconGenerationTarget) -> Result<()> {
    fs::create_dir_all(paths.generated_rs.parent().unwrap())?;
    fs::create_dir_all(paths.generated_catalog.parent().unwrap())?;

    write_if_changed(&paths.generated_rs, generated_module_source().as_bytes())?;
    if target == IconGenerationTarget::App {
        ensure_icons_rs(paths)?;
    }

    Ok(())
}

fn ensure_icons_rs(paths: &IconPaths) -> Result<()> {
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

fn check_framework_generated_integration(paths: &IconPaths, failures: &mut Vec<String>) {
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

fn generated_module_source() -> &'static str {
    "pub mod catalog;\npub mod symbols;\n"
}

fn read_manifest(path: &Path) -> Result<IconsManifest> {
    let contents = fs::read_to_string(path)?;
    let manifest: IconsManifest = toml::from_str(&contents)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_manifest(manifest: &IconsManifest) -> Result<()> {
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

fn validate_lucide_config(config: &LucideProviderConfig) -> Result<()> {
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

fn validate_manifest_ref(value: &str) -> Result<()> {
    ProviderRef::parse(value).map(|_| ())
}

fn validate_ref_custom_target(manifest: &IconsManifest, icon_ref: &ProviderRef) -> Result<()> {
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

fn fetch_lucide_svg(version: &str, slug: &str) -> Result<String> {
    let url =
        format!("https://raw.githubusercontent.com/lucide-icons/lucide/{version}/icons/{slug}.svg");

    fetch_text(&url)
}

fn read_custom_svg(paths: &IconPaths, manifest: &IconsManifest, name: &str) -> Result<String> {
    let Some(relative) = manifest.custom.get(name) else {
        return Err(format!("Custom icon `{name}` is not registered in [custom].").into());
    };

    let path = validate_custom_svg_path(paths, relative)?;
    fs::read_to_string(path).map_err(Into::into)
}

fn validate_custom_svg_path(paths: &IconPaths, relative: &str) -> Result<PathBuf> {
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

    Ok(path)
}

fn validate_manifest_relative_svg_path(relative: &str) -> Result<()> {
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

fn normalize_svg(source: &str, config: &LucideProviderConfig) -> Result<String> {
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

fn write_manifest(paths: &IconPaths, manifest: &IconsManifest) -> Result<()> {
    let updated = toml::to_string_pretty(manifest)?;
    write_if_changed(&paths.manifest, updated.as_bytes())?;
    Ok(())
}

fn write_generated_modules(
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

fn generate_catalog_source(
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

fn generate_symbols_source(
    manifest: &IconsManifest,
    target: IconGenerationTarget,
) -> Result<String> {
    let import = match target {
        IconGenerationTarget::App => "use nive::prelude::IconSource;",
        IconGenerationTarget::Framework => "use crate::icons::IconSource;",
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
    source.push_str("}\n");
    Ok(source)
}

fn include_path_for_ref(icon_ref: &ProviderRef) -> String {
    let path =
        PathBuf::from("../../../assets/icons/generated").join(icon_ref.generated_asset_path());
    path.to_string_lossy().replace('\\', "/")
}

fn collect_manifest_refs(manifest: &IconsManifest) -> Result<BTreeSet<ProviderRef>> {
    let mut refs = BTreeSet::new();

    for value in manifest.roles.values().chain(manifest.symbols.values()) {
        refs.insert(ProviderRef::parse(value)?);
    }

    Ok(refs)
}

fn remove_stale_assets(paths: &IconPaths, manifest: &IconsManifest) -> Result<()> {
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

fn collect_stale_assets(
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

fn walk_svg_files(root: &Path) -> Result<Vec<PathBuf>> {
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

fn check_generated_file(failures: &mut Vec<String>, path: &Path, expected: &[u8]) {
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

fn validate_role_name(name: &str) -> Result<()> {
    if role_variant(name).is_ok() {
        return Ok(());
    }

    Err(format!(
        "Unknown icon role `{name}`. Expected one of: {}",
        required_role_names().join(", ")
    )
    .into())
}

fn validate_variant(variant: &str) -> Result<()> {
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

fn validate_lucide_slug(slug: &str) -> Result<()> {
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

fn validate_ref_name(name: &str) -> Result<()> {
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

fn validate_stroke_line_value(field: &str, value: &str) -> Result<()> {
    match value {
        "butt" | "round" | "square" | "arcs" | "bevel" | "miter" | "miter-clip" => Ok(()),
        _ => Err(format!("Invalid {field} value (value: {value})").into()),
    }
}

fn ensure_provider(provider: &str) -> Result<()> {
    if provider == "lucide" {
        Ok(())
    } else {
        Err(format!("Unsupported icon provider `{provider}`. Supported providers: lucide.").into())
    }
}

fn default_role_refs() -> BTreeMap<String, String> {
    role_lucide_defaults()
        .iter()
        .map(|(role, slug, _variant)| ((*role).to_string(), format!("lucide:{slug}")))
        .collect()
}

fn required_role_names() -> Vec<&'static str> {
    role_lucide_defaults()
        .iter()
        .map(|(role, _slug, _variant)| *role)
        .collect()
}

fn role_variant(role: &str) -> Result<&'static str> {
    role_lucide_defaults()
        .iter()
        .find(|(candidate, _slug, _variant)| *candidate == role)
        .map(|(_role, _slug, variant)| *variant)
        .ok_or_else(|| format!("Unknown icon role `{role}`.").into())
}

fn role_lucide_defaults() -> &'static [(&'static str, &'static str, &'static str)] {
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
        ("view-conceal", "eye-off", "ViewConceal"),
        ("view-more", "ellipsis", "ViewMore"),
        ("view-refresh", "refresh-cw", "ViewRefresh"),
        ("view-reveal", "eye", "ViewReveal"),
        ("window-close", "x", "WindowClose"),
    ]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LucideMetadata {
    #[serde(default)]
    cache_version: u8,
    icons: Vec<LucideIconMetadata>,
}

impl LucideMetadata {
    const CACHE_VERSION: u8 = 1;

    fn search(&self, query: &str) -> Vec<&LucideIconMetadata> {
        self.icons
            .iter()
            .filter(|icon| icon.matches(query))
            .collect()
    }

    fn list(&self, category: Option<&str>) -> Vec<&LucideIconMetadata> {
        self.icons
            .iter()
            .filter(|icon| {
                category.is_none_or(|category| {
                    icon.categories
                        .iter()
                        .any(|candidate| candidate.eq_ignore_ascii_case(category))
                })
            })
            .collect()
    }

    fn find(&self, slug: &str) -> Option<&LucideIconMetadata> {
        self.icons.iter().find(|icon| icon.name == slug)
    }

    #[cfg(test)]
    fn fallback() -> Self {
        let mut icons: Vec<_> = role_lucide_defaults()
            .iter()
            .map(|(_role, slug, _variant)| LucideIconMetadata {
                name: (*slug).to_string(),
                aliases: Vec::new(),
                tags: Vec::new(),
                categories: vec!["nive-default".to_string()],
                use_cases: vec!["framework role default".to_string()],
            })
            .collect();

        icons.extend([
            LucideIconMetadata {
                name: "arrow-up".to_string(),
                aliases: vec!["up".to_string()],
                tags: vec!["north".to_string(), "direction".to_string()],
                categories: vec!["arrows".to_string(), "navigation".to_string()],
                use_cases: vec!["sort ascending".to_string()],
            },
            LucideIconMetadata {
                name: "user".to_string(),
                aliases: vec!["account".to_string(), "person".to_string()],
                tags: vec!["profile".to_string(), "avatar".to_string()],
                categories: vec!["users".to_string()],
                use_cases: vec!["account menu".to_string()],
            },
            LucideIconMetadata {
                name: "shield-check".to_string(),
                aliases: vec!["security".to_string()],
                tags: vec!["safe".to_string(), "verified".to_string()],
                categories: vec!["security".to_string()],
                use_cases: vec!["verified state".to_string()],
            },
        ]);

        icons.sort_by(|left, right| left.name.cmp(&right.name));
        icons.dedup_by(|left, right| left.name == right.name);
        Self {
            cache_version: Self::CACHE_VERSION,
            icons,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LucideIconMetadata {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default, rename = "useCases")]
    use_cases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LucideIconMetadataFile {
    #[serde(default, deserialize_with = "deserialize_string_values")]
    aliases: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_values")]
    tags: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_values")]
    categories: Vec<String>,
    #[serde(
        default,
        rename = "useCases",
        deserialize_with = "deserialize_string_values"
    )]
    use_cases: Vec<String>,
}

impl LucideIconMetadata {
    fn matches(&self, query: &str) -> bool {
        self.name.contains(query)
            || self.aliases.iter().any(|value| contains(value, query))
            || self.tags.iter().any(|value| contains(value, query))
            || self.categories.iter().any(|value| contains(value, query))
            || self.use_cases.iter().any(|value| contains(value, query))
    }

    fn summary_line(&self) -> String {
        let mut parts = vec![format!("lucide:{}", self.name)];

        if !self.categories.is_empty() {
            parts.push(format!("categories={}", self.categories.join(",")));
        }
        if !self.tags.is_empty() {
            parts.push(format!("tags={}", self.tags.join(",")));
        }
        if !self.aliases.is_empty() {
            parts.push(format!("aliases={}", self.aliases.join(",")));
        }

        parts.join("  ")
    }
}

struct LucideProvider<'a> {
    paths: &'a IconPaths,
    version: &'a str,
}

impl<'a> LucideProvider<'a> {
    fn new(paths: &'a IconPaths, version: &'a str) -> Self {
        Self { paths, version }
    }

    fn metadata(&self) -> Result<LucideMetadata> {
        fs::create_dir_all(&self.paths.metadata_cache_dir)?;
        let cache_path = self
            .paths
            .metadata_cache_dir
            .join(format!("lucide-{}.json", self.version));

        if cache_path.exists() {
            let contents = fs::read_to_string(&cache_path)?;
            let metadata: LucideMetadata = serde_json::from_str(&contents)?;
            if metadata.cache_version == LucideMetadata::CACHE_VERSION {
                return Ok(metadata);
            }
        }

        let metadata = self.fetch_metadata()?;
        fs::write(&cache_path, serde_json::to_string_pretty(&metadata)?)?;
        Ok(metadata)
    }

    fn fetch_metadata(&self) -> Result<LucideMetadata> {
        let url = format!(
            "https://codeload.github.com/lucide-icons/lucide/tar.gz/{}",
            self.version
        );
        let archive_bytes = fetch_bytes(&url)?;
        let decoder = GzDecoder::new(archive_bytes.as_slice());
        let mut archive = Archive::new(decoder);
        let mut icons = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let Some(slug) = lucide_metadata_slug(&path) else {
                continue;
            };

            let mut contents = String::new();
            entry.read_to_string(&mut contents)?;
            let file: LucideIconMetadataFile =
                serde_json::from_str(&contents).map_err(|error| {
                    format!("Failed to parse Lucide metadata for `{slug}` from {url}: {error}")
                })?;
            icons.push(LucideIconMetadata {
                name: slug,
                aliases: file.aliases,
                tags: file.tags,
                categories: file.categories,
                use_cases: file.use_cases,
            });
        }

        icons.sort_by(|left, right| left.name.cmp(&right.name));
        icons.dedup_by(|left, right| left.name == right.name);
        if icons.is_empty() {
            return Err(format!(
                "Lucide metadata archive for {} contained no icons.",
                self.version
            )
            .into());
        }

        Ok(LucideMetadata {
            cache_version: LucideMetadata::CACHE_VERSION,
            icons,
        })
    }
}

fn fetch_text(url: &str) -> Result<String> {
    let mut response = ureq::get(url)
        .header("User-Agent", "nive-cli")
        .call()
        .map_err(|error| format!("Failed to fetch {url}: {error}"))?;

    response
        .body_mut()
        .read_to_string()
        .map_err(|error| format!("Failed to read response for {url}: {error}").into())
}

fn fetch_bytes(url: &str) -> Result<Vec<u8>> {
    let mut response = ureq::get(url)
        .header("User-Agent", "nive-cli")
        .call()
        .map_err(|error| format!("Failed to fetch {url}: {error}"))?;

    response
        .body_mut()
        .read_to_vec()
        .map_err(|error| format!("Failed to read response for {url}: {error}").into())
}

fn lucide_metadata_slug(path: &Path) -> Option<String> {
    let mut components = path.components();
    components.next()?;

    if components.next()?.as_os_str() != OsStr::new("icons") {
        return None;
    }

    let file_name = components.next()?.as_os_str().to_str()?;
    if components.next().is_some() {
        return None;
    }

    file_name.strip_suffix(".json").map(str::to_string)
}

fn deserialize_string_values<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let mut values = Vec::new();
    collect_string_values(&value, &mut values);
    Ok(values)
}

fn collect_string_values(value: &serde_json::Value, values: &mut Vec<String>) {
    match value {
        serde_json::Value::String(value) => values.push(value.clone()),
        serde_json::Value::Array(items) => {
            for item in items {
                collect_string_values(item, values);
            }
        }
        serde_json::Value::Object(fields) => {
            if let Some(serde_json::Value::String(name)) = fields.get("name") {
                values.push(name.clone());
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

fn contains(value: &str, query: &str) -> bool {
    value.to_ascii_lowercase().contains(query)
}

fn print_metadata_list(label: &str, values: &[String]) {
    if !values.is_empty() {
        println!("{label}: {}", values.join(", "));
    }
}

fn generate_gallery_html(metadata: &LucideMetadata) -> String {
    let mut html = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Lucide Icon Gallery</title>\
         <style>body{font:14px system-ui,sans-serif;margin:24px;color:#17202a}\
         .grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:12px}\
         .item{border:1px solid #d7dde5;border-radius:6px;padding:12px}\
         code{font-size:12px;color:#425466}</style></head><body>\
         <h1>Lucide Icon Gallery</h1><div class=\"grid\">",
    );

    for icon in &metadata.icons {
        html.push_str("<div class=\"item\"><strong>");
        html.push_str(&escape_html(&icon.name));
        html.push_str("</strong><br><code>lucide:");
        html.push_str(&escape_html(&icon.name));
        html.push_str("</code>");
        if !icon.categories.is_empty() {
            html.push_str("<br>");
            html.push_str(&escape_html(&icon.categories.join(", ")));
        }
        html.push_str("</div>");
    }

    html.push_str("</div></body></html>\n");
    html
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn open_path(path: &Path) -> Result<()> {
    let status = if cfg!(target_os = "macos") {
        Command::new("open").arg(path).status()?
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg(path)
            .status()?
    } else {
        Command::new("xdg-open").arg(path).status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!("Failed to open {}", display_path(path)).into())
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

    fn manifest_with_refs(
        roles: &[(&str, &str)],
        symbols: &[(&str, &str)],
        custom: &[(&str, &str)],
    ) -> IconsManifest {
        let mut manifest = empty_manifest();
        manifest.roles = roles
            .iter()
            .map(|(role, value)| ((*role).to_string(), (*value).to_string()))
            .collect();
        manifest.symbols = symbols
            .iter()
            .map(|(symbol, value)| ((*symbol).to_string(), (*value).to_string()))
            .collect();
        manifest.custom = custom
            .iter()
            .map(|(name, path)| ((*name).to_string(), (*path).to_string()))
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

    fn write_svg(path: &Path) {
        fs::create_dir_all(path.parent().unwrap()).expect("create svg parent");
        fs::write(path, "<svg><path d=\"M1 1\" /></svg>\n").expect("write svg");
    }

    #[test]
    fn path_planning_uses_provider_neutral_icon_locations() {
        let root = Path::new("/tmp/demo-app");
        let paths = IconPaths::from_root(root);

        assert_eq!(paths.manifest, root.join("icons.toml"));
        assert_eq!(
            paths.generated_asset_dir,
            root.join("assets/icons/generated")
        );
        assert_eq!(paths.icons_rs, root.join("src/icons.rs"));
        assert_eq!(paths.generated_rs, root.join("src/icons/generated.rs"));
        assert_eq!(
            paths.generated_catalog,
            root.join("src/icons/generated/catalog.rs")
        );
        assert_eq!(
            paths.generated_symbols,
            root.join("src/icons/generated/symbols.rs")
        );
    }

    #[test]
    fn manifest_validation_rejects_unqualified_refs() {
        let manifest = manifest_with_refs(&[("window-close", "x")], &[], &[]);

        let error = validate_manifest(&manifest).expect_err("unqualified ref should fail");

        assert!(
            error.to_string().contains("missing a provider"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn command_ref_input_normalizes_bare_lucide_refs() {
        let icon_ref = ProviderRef::parse_command_input("x").expect("parse shorthand");

        assert_eq!(icon_ref.normalized(), "lucide:x");
    }

    #[test]
    fn source_generation_uses_named_modules_and_icon_symbol() {
        let manifest = manifest_with_refs(
            &[("window-close", "lucide:x")],
            &[("User", "lucide:user")],
            &[],
        );

        let catalog =
            generate_catalog_source(&manifest, IconGenerationTarget::App).expect("catalog source");
        let symbols =
            generate_symbols_source(&manifest, IconGenerationTarget::App).expect("symbols source");

        assert!(catalog.contains("pub const APP_ICON_CATALOG: IconCatalog"));
        assert!(catalog.contains("IconRole::WindowClose"));
        assert!(symbols.contains("pub enum IconSymbol"));
        assert!(symbols.contains("impl IconSource for IconSymbol"));
        assert!(symbols.contains("lucide:user"));
    }

    #[test]
    fn generated_module_integration_files_are_created_without_network() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let paths = IconPaths::from_root(tempdir.path());

        icons_init(&paths).expect("init icons");

        assert!(fs::read_to_string(&paths.icons_rs)
            .expect("read icons rs")
            .contains("APP_ICON_CATALOG"));
        assert_eq!(
            fs::read_to_string(&paths.generated_rs).expect("read generated rs"),
            generated_module_source()
        );
        assert!(fs::read_to_string(&paths.generated_symbols)
            .expect("read symbols")
            .contains("pub enum IconSymbol"));
    }

    #[test]
    fn init_preserves_user_authored_icons_exports() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let paths = IconPaths::from_root(tempdir.path());
        fs::create_dir_all(paths.icons_rs.parent().unwrap()).expect("create src");
        fs::write(&paths.icons_rs, "pub use custom::BrandMark;\n").expect("write custom exports");

        icons_init(&paths).expect("init icons");

        let contents = fs::read_to_string(&paths.icons_rs).expect("read icons rs");
        assert!(contents.contains("pub use custom::BrandMark;"));
        assert!(contents.contains("mod generated;"));
        assert!(contents.contains("IconSymbol"));
    }

    #[test]
    fn custom_svg_paths_cannot_escape_app_root() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let paths = IconPaths::from_root(tempdir.path());

        let error =
            validate_custom_svg_path(&paths, "../escape.svg").expect_err("path escape should fail");

        assert!(
            error.to_string().contains("cannot contain `..`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn custom_svg_sync_and_check_are_offline() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let paths = IconPaths::from_root(tempdir.path());
        write_svg(&tempdir.path().join("assets/icons/custom/brand-mark.svg"));

        let manifest = manifest_with_refs(
            &default_role_refs()
                .iter()
                .map(|(role, value)| (role.as_str(), value.as_str()))
                .collect::<Vec<_>>(),
            &[("BrandMark", "custom:brand-mark")],
            &[("brand-mark", "assets/icons/custom/brand-mark.svg")],
        );
        write_manifest(&paths.manifest, &manifest);
        ensure_icon_module_files(&paths, IconGenerationTarget::App).expect("ensure modules");

        for icon_ref in collect_manifest_refs(&manifest).expect("collect refs") {
            let source = match &icon_ref {
                ProviderRef::Lucide(_) => "<svg><path d=\"M2 2\" /></svg>\n".to_string(),
                ProviderRef::Custom(name) => read_custom_svg(&paths, &manifest, name)
                    .expect("read custom svg for generated asset"),
            };
            let normalized = normalize_svg(&source, &manifest.provider.lucide).expect("normalize");
            write_if_changed(
                &paths
                    .generated_asset_dir
                    .join(icon_ref.generated_asset_path()),
                normalized.as_bytes(),
            )
            .expect("write generated asset");
        }
        write_generated_modules(&paths, &manifest, IconGenerationTarget::App)
            .expect("write generated");

        icons_check(&paths, IconGenerationTarget::App).expect("offline check");
    }

    #[test]
    fn check_reports_missing_required_role_coverage() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let paths = IconPaths::from_root(tempdir.path());
        write_manifest(
            &paths.manifest,
            &manifest_with_refs(&[("window-close", "lucide:x")], &[], &[]),
        );

        let error =
            icons_check(&paths, IconGenerationTarget::App).expect_err("missing roles should fail");

        assert!(
            error.to_string().contains("Icon check failed"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn provider_discovery_uses_fixture_metadata_for_search_list_show_and_gallery() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let paths = IconPaths::from_root(tempdir.path());
        let metadata = LucideMetadata::fallback();
        fs::create_dir_all(&paths.metadata_cache_dir).expect("create metadata cache");
        fs::write(
            paths.metadata_cache_dir.join("lucide-0.460.0.json"),
            serde_json::to_string_pretty(&metadata).expect("serialize metadata"),
        )
        .expect("write fixture metadata cache");

        assert!(metadata
            .search("profile")
            .iter()
            .any(|icon| icon.name == "user"));
        assert!(metadata
            .list(Some("security"))
            .iter()
            .any(|icon| icon.name == "shield-check"));
        assert!(metadata
            .list(Some("arrows"))
            .iter()
            .any(|icon| icon.name == "arrow-up"));
        assert!(metadata.find("arrow-up").is_some());
        assert!(metadata
            .search("sort ascending")
            .iter()
            .any(|icon| icon.name == "arrow-up"));

        let html = generate_gallery_html(&metadata);
        assert!(html.contains("lucide:user"));

        let provider = LucideProvider::new(&paths, "0.460.0");
        assert!(provider
            .metadata()
            .expect("metadata")
            .find("user")
            .is_some());
    }

    #[test]
    fn command_check_returns_legacy_manifest_migration_error() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(tempdir.path().join("icons")).expect("create icons dir");
        fs::write(tempdir.path().join("icons/lucide.toml"), "").expect("write legacy manifest");

        let error = run_in_dir(IconsCommands::Check { framework: false }, tempdir.path())
            .expect_err("missing manifest fails");

        assert!(
            error.to_string().contains("legacy"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn framework_generation_uses_crate_imports() {
        let manifest = manifest_with_refs(&[("window-close", "lucide:x")], &[], &[]);

        let catalog = generate_catalog_source(&manifest, IconGenerationTarget::Framework)
            .expect("catalog source");
        let symbols = generate_symbols_source(&manifest, IconGenerationTarget::Framework)
            .expect("symbols source");

        assert!(catalog.contains("use crate::icons::{"));
        assert!(!catalog.contains("nive::prelude"));
        assert!(symbols.contains("use crate::icons::IconSource;"));
    }
}

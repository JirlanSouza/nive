use std::{fs, path::Path};

use super::generate::{
    check_framework_generated_integration, check_generated_file, collect_manifest_refs,
    collect_stale_assets, ensure_icon_module_files, generate_catalog_source,
    generate_symbols_source, generated_module_source, normalize_svg, remove_stale_assets,
    write_generated_modules, write_if_changed,
};
use super::lucide::{
    display_path, fetch_lucide_svg, generate_gallery_html, open_path, print_metadata_list,
};
use super::manifest::{
    empty_manifest, ensure_provider, read_custom_svg, read_manifest, require_manifest,
    required_role_names, validate_custom_svg_path, validate_ref_custom_target, validate_ref_name,
    validate_role_name, validate_variant, write_manifest,
};
use super::{
    IconGenerationTarget, IconPaths, IconsCommands, IconsManifest, LucideProvider,
    LucideProviderConfig, ProviderRef, Result,
};

pub(super) fn run_in_dir(command: IconsCommands, root: &Path) -> Result<()> {
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
            framework,
        } => icons_add_symbol(
            &paths,
            &variant,
            &provider_ref,
            generation_target(framework),
        ),
        IconsCommands::SetRole {
            role_name,
            provider_ref,
            framework,
        } => icons_set_role(
            &paths,
            &role_name,
            &provider_ref,
            generation_target(framework),
        ),
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

pub(super) fn generation_target(framework: bool) -> IconGenerationTarget {
    if framework {
        IconGenerationTarget::Framework
    } else {
        IconGenerationTarget::App
    }
}

pub(super) fn icons_list_manifest(paths: &IconPaths) -> Result<()> {
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

pub(super) fn icons_sync(paths: &IconPaths, target: IconGenerationTarget) -> Result<()> {
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

pub(super) fn icons_check(paths: &IconPaths, target: IconGenerationTarget) -> Result<()> {
    require_manifest(paths)?;

    let manifest = read_manifest(&paths.manifest)?;
    let mut failures = Vec::new();

    failures.extend(missing_required_role_failures(paths, &manifest));

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

pub(super) fn missing_required_role_failures(
    paths: &IconPaths,
    manifest: &IconsManifest,
) -> Vec<String> {
    required_role_names()
        .into_iter()
        .filter(|role| !manifest.roles.contains_key(*role))
        .map(|role| {
            format!(
                "{} is missing required icon role `{role}`. Add a provider mapping under `[roles]` and run `nive icons sync`.",
                display_path(&paths.manifest)
            )
        })
        .collect()
}

pub(super) fn icons_add_symbol(
    paths: &IconPaths,
    variant: &str,
    value: &str,
    target: IconGenerationTarget,
) -> Result<()> {
    validate_variant(variant)?;
    require_manifest(paths)?;

    let icon_ref = ProviderRef::parse_command_input(value)?;
    let mut manifest = read_manifest(&paths.manifest)?;
    validate_ref_custom_target(&manifest, &icon_ref)?;
    manifest
        .symbols
        .insert(variant.to_string(), icon_ref.normalized());

    write_manifest(paths, &manifest)?;
    icons_sync(paths, target)
}

pub(super) fn icons_set_role(
    paths: &IconPaths,
    role_name: &str,
    value: &str,
    target: IconGenerationTarget,
) -> Result<()> {
    validate_role_name(role_name)?;
    require_manifest(paths)?;

    let icon_ref = ProviderRef::parse_command_input(value)?;
    let mut manifest = read_manifest(&paths.manifest)?;
    validate_ref_custom_target(&manifest, &icon_ref)?;
    manifest
        .roles
        .insert(role_name.to_string(), icon_ref.normalized());

    write_manifest(paths, &manifest)?;
    icons_sync(paths, target)
}

pub(super) fn icons_add_custom(paths: &IconPaths, name: &str, source_path: &str) -> Result<()> {
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

pub(super) fn icons_init(paths: &IconPaths) -> Result<()> {
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

pub(super) fn icons_search(paths: &IconPaths, provider: &str, query: &str) -> Result<()> {
    ensure_provider(provider)?;
    let version = lucide_provider_version(paths)?;
    let metadata = LucideProvider::new(paths, &version).metadata()?;
    let query = query.to_ascii_lowercase();

    for icon in metadata.search(&query) {
        println!("{}", icon.summary_line());
    }

    Ok(())
}

pub(super) fn icons_provider_list(
    paths: &IconPaths,
    provider: &str,
    category: Option<&str>,
) -> Result<()> {
    ensure_provider(provider)?;
    let version = lucide_provider_version(paths)?;
    let metadata = LucideProvider::new(paths, &version).metadata()?;

    for icon in metadata.list(category) {
        println!("{}", icon.summary_line());
    }

    Ok(())
}

pub(super) fn icons_show(paths: &IconPaths, value: &str) -> Result<()> {
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

pub(super) fn icons_gallery(paths: &IconPaths, provider: &str, open: bool) -> Result<()> {
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

pub(super) fn lucide_provider_version(paths: &IconPaths) -> Result<String> {
    if paths.manifest.exists() {
        return Ok(read_manifest(&paths.manifest)?.provider.lucide.version);
    }

    Ok(LucideProviderConfig::default().version)
}

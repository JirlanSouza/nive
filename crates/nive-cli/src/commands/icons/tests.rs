use std::fs;
use std::path::Path;

use super::commands::*;
use super::generate::*;
use super::lucide::*;
use super::manifest::*;
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
    fs::write(
        path,
        concat!(
            "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" ",
            "stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
            "<path d=\"M4 4L20 20\" /></svg>\n"
        ),
    )
    .expect("write svg");
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
fn custom_svg_contract_rejects_non_monochrome_source_offline() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let paths = IconPaths::from_root(tempdir.path());
    let source = tempdir.path().join("assets/icons/custom/bad.svg");
    fs::create_dir_all(source.parent().unwrap()).expect("create custom icon parent");
    fs::write(
        &source,
        concat!(
            "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" ",
            "stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
            "<path fill=\"#ff0000\" d=\"M4 4H20V20H4Z\" /></svg>\n"
        ),
    )
    .expect("write invalid custom icon");

    let error = validate_custom_svg_path(&paths, "assets/icons/custom/bad.svg")
        .expect_err("fixed-color custom icon should fail");

    assert!(
        error.to_string().contains("monochrome"),
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
    write_generated_modules(&paths, &manifest, IconGenerationTarget::App).expect("write generated");

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
fn required_role_coverage_includes_identity() {
    assert!(required_role_names().contains(&"identity"));
    assert_eq!(
        role_variant("identity").expect("identity variant"),
        "Identity"
    );
}

#[test]
fn required_role_coverage_includes_validation_error_and_identity() {
    assert!(required_role_names().contains(&"validation-error"));
    assert!(required_role_names().contains(&"identity"));
    assert_eq!(
        role_variant("validation-error").expect("validation-error variant"),
        "ValidationError"
    );
}

#[test]
fn missing_validation_error_has_an_actionable_offline_diagnostic() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let paths = IconPaths::from_root(tempdir.path());
    let mut manifest = empty_manifest();
    manifest.roles = default_role_refs();
    manifest.roles.remove("validation-error");

    let failures = missing_required_role_failures(&paths, &manifest);

    assert_eq!(failures.len(), 1);
    assert!(failures[0].contains("`validation-error`"));
    assert!(failures[0].contains("[roles]"));
    assert!(failures[0].contains("nive icons sync"));
    assert!(manifest.roles.contains_key("identity"));
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

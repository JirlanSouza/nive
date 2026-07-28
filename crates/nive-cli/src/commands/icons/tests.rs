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
        &[("window-close", "lucide:x"), ("identity", "lucide:user")],
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
fn an_app_manifest_is_additive_over_the_framework_catalog() {
    // Requiring the full list would mean every new `IconRole` invalidated every
    // existing app.
    let tempdir = tempfile::tempdir().expect("tempdir");
    let paths = IconPaths::from_root(tempdir.path());
    write_manifest(&paths.manifest, &manifest_with_refs(&[], &[], &[]));

    icons_sync(&paths, IconGenerationTarget::App).expect("an empty manifest syncs");

    icons_check(&paths, IconGenerationTarget::App)
        .expect("an app declaring no roles at all is valid");
}

#[test]
fn semantic_roles_keep_their_own_variants() {
    // Both resolve to a Lucide glyph another role also uses; that must not
    // collapse them into one variant.
    assert_eq!(
        role_variant("identity").expect("identity variant"),
        "Identity"
    );
    assert_eq!(
        role_variant("validation-error").expect("validation-error variant"),
        "ValidationError"
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
    assert!(symbols.contains("use crate::icons::{IconRef, IconSource};"));
    // A generated symbol must drop straight into a widget icon slot.
    assert!(symbols.contains("impl From<IconSymbol> for IconRef"));
}

/// The roles `IconRole` declares, read from `nive-ui`'s source because the CLI
/// does not depend on it. Test-only: nothing in production reads this.
fn framework_roles() -> Vec<(String, String)> {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../nive-ui/src/icons.rs"))
            .expect("nive-ui icons.rs is readable from the workspace");
    let body = source
        .split_once("fn canonical_name")
        .expect("canonical_name exists")
        .1;
    let body = body.split_once("\n    }").expect("its match ends").0;

    body.lines()
        .filter_map(|line| line.split_once("=> \""))
        .filter_map(|(variant, rest)| {
            let name = rest.split_once('"')?.0;
            let variant = variant.trim().strip_prefix("Self::")?.trim();
            Some((name.to_string(), variant.to_string()))
        })
        .collect()
}

#[test]
fn deriving_the_variant_agrees_with_every_role_the_framework_declares() {
    // Spelling the variant instead of looking it up is only safe if the rule
    // matches what `IconRole` declares.
    let roles = framework_roles();

    assert!(
        !roles.is_empty(),
        "failed to read any role from nive-ui; this test's parser needs updating"
    );

    for (name, expected) in roles {
        assert_eq!(
            role_variant(&name).expect("a framework role name is well formed"),
            expected,
            "derived variant for `{name}` does not match IconRole"
        );
    }
}

#[test]
fn a_role_the_cli_has_never_heard_of_still_generates() {
    // An app pinned to a newer `nive` than the CLI build. The CLI must spell the
    // role and let the app's compiler judge it.
    let manifest = manifest_with_refs(&[("view-hypothetical", "lucide:x")], &[], &[]);

    let catalog = generate_catalog_source(&manifest, IconGenerationTarget::App)
        .expect("an unrecognised role is not the CLI's to reject");

    assert!(
        catalog.contains("IconRole::ViewHypothetical"),
        "expected the derived variant in:\n{catalog}"
    );
}

#[test]
fn a_malformed_role_name_is_rejected_but_an_unknown_one_is_not() {
    // Form is what this tool can see; existence is the app compiler's question.
    for malformed in [
        "View-Theme",
        "view_theme",
        "",
        "view--theme",
        "2fast",
        "vïew",
    ] {
        assert!(
            validate_role_name(malformed).is_err(),
            "`{malformed}` is not a legal role name"
        );
    }

    validate_role_name("view-hypothetical").expect("an unknown but well-formed role is accepted");
}

#[test]
fn init_and_scaffold_agree_on_the_starting_manifest() {
    // `nive icons init` and `nive new` are two doors into the same project
    // shape. Both must hand the author an additive manifest rather than a copy
    // of the framework's role list.
    let tempdir = tempfile::tempdir().expect("tempdir");
    let paths = IconPaths::from_root(tempdir.path());

    icons_init(&paths).expect("init icons");

    let manifest = read_manifest(&paths.manifest).expect("read manifest");
    assert!(
        manifest.roles.is_empty(),
        "init must not pre-declare framework roles, got {:?}",
        manifest.roles.keys().collect::<Vec<_>>()
    );

    icons_check(&paths, IconGenerationTarget::App).expect("a freshly initialised app checks clean");
}

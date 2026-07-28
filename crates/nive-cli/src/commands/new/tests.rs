use super::*;

#[test]
fn title_case_converts_snake_case() {
    assert_eq!(to_title_case("my_app"), "MyApp");
}

#[test]
fn title_case_converts_kebab_case() {
    assert_eq!(to_title_case("my-app"), "MyApp");
}

#[test]
fn title_case_handles_single_word() {
    assert_eq!(to_title_case("hello"), "Hello");
}

#[test]
fn title_case_handles_already_titled() {
    assert_eq!(to_title_case("Hello"), "Hello");
}

#[test]
fn title_case_handles_multiple_separators() {
    assert_eq!(to_title_case("my_cool_app"), "MyCoolApp");
}

// --- build_nive_dep ---

#[test]
fn dep_crates_io_basic() {
    let dep = build_nive_dep(None, None, None, None, false).unwrap();
    assert_eq!(dep, r#"nive = "0.1""#);
}

#[test]
fn dep_crates_io_dashboard() {
    let dep = build_nive_dep(None, None, None, None, true).unwrap();
    assert_eq!(
        dep,
        r#"nive = { version = "0.1", features = ["file-picker"] }"#
    );
}

#[test]
fn dep_git_tag_basic() {
    let dep = build_nive_dep(
        Some("https://github.com/JirlanSouza/nive"),
        Some("v0.1.0-alpha.1"),
        None,
        None,
        false,
    )
    .unwrap();
    assert_eq!(
        dep,
        r#"nive = { git = "https://github.com/JirlanSouza/nive", tag = "v0.1.0-alpha.1" }"#
    );
}

#[test]
fn dep_git_tag_dashboard() {
    let dep = build_nive_dep(
        Some("https://github.com/JirlanSouza/nive"),
        Some("v0.1.0-alpha.1"),
        None,
        None,
        true,
    )
    .unwrap();
    assert_eq!(
        dep,
        r#"nive = { git = "https://github.com/JirlanSouza/nive", tag = "v0.1.0-alpha.1", features = ["file-picker"] }"#
    );
}

#[test]
fn dep_git_rev_basic() {
    let dep = build_nive_dep(
        Some("https://github.com/JirlanSouza/nive"),
        None,
        Some("abc1234"),
        None,
        false,
    )
    .unwrap();
    assert_eq!(
        dep,
        r#"nive = { git = "https://github.com/JirlanSouza/nive", rev = "abc1234" }"#
    );
}

#[test]
fn dep_git_branch_basic() {
    let dep = build_nive_dep(
        Some("https://github.com/JirlanSouza/nive"),
        None,
        None,
        Some("main"),
        false,
    )
    .unwrap();
    assert_eq!(
        dep,
        r#"nive = { git = "https://github.com/JirlanSouza/nive", branch = "main" }"#
    );
}

#[test]
fn dep_git_without_ref_is_error() {
    let err = build_nive_dep(
        Some("https://github.com/JirlanSouza/nive"),
        None,
        None,
        None,
        false,
    )
    .unwrap_err();
    assert!(err.to_string().contains("--git requires exactly one"));
}

#[test]
fn dep_ref_without_git_is_error() {
    let err = build_nive_dep(None, Some("v0.1.0-alpha.1"), None, None, false).unwrap_err();
    assert!(err.to_string().contains("require --git"));
}

#[test]
fn dep_multiple_refs_is_error() {
    let err = build_nive_dep(
        Some("https://github.com/JirlanSouza/nive"),
        Some("v0.1.0-alpha.1"),
        Some("abc1234"),
        None,
        false,
    )
    .unwrap_err();
    assert!(err.to_string().contains("exactly one"));
}

fn collect_relative_files(dir: &Dir) -> Vec<String> {
    let mut out: Vec<String> = dir
        .files()
        .map(|f| f.path().to_string_lossy().into_owned())
        .collect();
    for subdir in dir.dirs() {
        let prefix = subdir.path().to_string_lossy().into_owned();
        for f in subdir.files() {
            let p = format!(
                "{}/{}",
                prefix,
                f.path().file_name().unwrap().to_string_lossy()
            );
            out.push(p);
        }
    }
    out.sort();
    out
}

#[test]
fn copy_templates_substitutes_placeholders_and_strips_template_suffix() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let app_dir = tempdir.path().join("my-app");
    let nive_dep = r#"nive = "0.1""#;

    copy_templates(&BASIC_TEMPLATES, &app_dir, "my_app", "MyApp", nive_dep).expect("copy");

    let main_path = app_dir.join("src/main.rs");
    assert!(
        main_path.exists(),
        "main.rs should be written without .template suffix"
    );

    let main_contents = fs::read_to_string(&main_path).expect("read main.rs");
    assert!(
        main_contents.contains("struct MyApp"),
        "title placeholder not replaced"
    );
    assert!(
        !main_contents.contains("{{app_name_title}}"),
        "title placeholder left behind"
    );
    assert!(
        main_contents.contains(r#"ApplicationConfig::new("my_app")"#),
        "app_name placeholder not replaced"
    );
    assert!(
        !main_contents.contains("{{app_name}}"),
        "app_name placeholder left behind"
    );

    let toml = fs::read_to_string(app_dir.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(toml.starts_with("[package]\nname = \"my_app\""));
    assert!(toml.contains(nive_dep), "nive_dep placeholder not replaced");
    assert!(
        !toml.contains("{{nive_dep}}"),
        "nive_dep placeholder left behind"
    );
}

#[test]
fn copy_templates_writes_all_basic_template_files() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let app_dir = tempdir.path().join("basic-app");

    copy_templates(
        &BASIC_TEMPLATES,
        &app_dir,
        "basic_app",
        "BasicApp",
        r#"nive = "0.1""#,
    )
    .expect("copy");

    let expected = collect_relative_files(&BASIC_TEMPLATES)
        .into_iter()
        .map(|p| p.replace(".template", ""))
        .collect::<Vec<_>>();
    for rel in expected {
        let path = app_dir.join(&rel);
        assert!(path.exists(), "missing {} after copy", rel);
    }
}

#[test]
fn scaffold_basic_git_tag() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let app_dir = tempdir.path().join("git-tag-app");
    let nive_dep =
        r#"nive = { git = "https://github.com/JirlanSouza/nive", tag = "v0.1.0-alpha.1" }"#;

    copy_templates(
        &BASIC_TEMPLATES,
        &app_dir,
        "git_tag_app",
        "GitTagApp",
        nive_dep,
    )
    .expect("copy");

    let toml = fs::read_to_string(app_dir.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(toml.contains(nive_dep), "git tag dep not in Cargo.toml");
}

#[test]
fn scaffold_dashboard_git_tag() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let app_dir = tempdir.path().join("dashboard-git-tag-app");
    let nive_dep = r#"nive = { git = "https://github.com/JirlanSouza/nive", tag = "v0.1.0-alpha.1", features = ["file-picker"] }"#;

    copy_templates(
        &DASHBOARD_TEMPLATES,
        &app_dir,
        "dashboard_git_tag_app",
        "DashboardGitTagApp",
        nive_dep,
    )
    .expect("copy");

    let toml = fs::read_to_string(app_dir.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(
        toml.contains(nive_dep),
        "dashboard git tag dep not in Cargo.toml"
    );
}

#[test]
fn scaffold_basic_git_rev() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let app_dir = tempdir.path().join("git-rev-app");
    let nive_dep = r#"nive = { git = "https://github.com/JirlanSouza/nive", rev = "abc1234" }"#;

    copy_templates(
        &BASIC_TEMPLATES,
        &app_dir,
        "git_rev_app",
        "GitRevApp",
        nive_dep,
    )
    .expect("copy");

    let toml = fs::read_to_string(app_dir.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(toml.contains(nive_dep), "git rev dep not in Cargo.toml");
}

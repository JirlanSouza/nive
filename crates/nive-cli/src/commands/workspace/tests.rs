use std::{fs, path::Path};

use super::*;
use crate::commands::new;

/// A workspace root whose `members` array is written exactly as given.
fn workspace_root(members: &str) -> tempfile::TempDir {
    let tempdir = tempfile::tempdir().expect("tempdir");
    fs::write(
        tempdir.path().join("Cargo.toml"),
        format!("[workspace]\nmembers = {members}\nresolver = \"2\"\n"),
    )
    .expect("write workspace manifest");
    tempdir
}

fn scaffold(parent: &Path, name: &str) {
    new::run_in_dir(parent, name, false, None, None, None, None).expect("scaffold");
}

fn members_of(root: &Path) -> String {
    fs::read_to_string(root.join("Cargo.toml")).expect("read workspace manifest")
}

#[test]
fn a_new_app_inside_a_workspace_is_registered() {
    let workspace = workspace_root("[\"crates/existing\"]");
    scaffold(workspace.path(), "my-app");

    assert!(
        members_of(workspace.path()).contains("\"my-app\""),
        "the new crate must be a member, or Cargo refuses to build it:\n{}",
        members_of(workspace.path())
    );
}

#[test]
fn registration_reports_the_manifest_it_touched() {
    // Writing outside the directory the user named is the surprise this reports.
    let workspace = workspace_root("[]");
    let app = workspace.path().join("my-app");
    fs::create_dir_all(&app).expect("create app dir");

    let registration = register_member(&app).expect("register");

    let report = registration.report().expect("a modified file is reported");
    assert!(report.contains("my-app"), "unexpected report: {report}");
    assert!(report.contains("Cargo.toml"), "unexpected report: {report}");
    assert!(matches!(registration, Registration::Registered { .. }));
}

#[test]
fn a_second_registration_is_a_no_op() {
    let workspace = workspace_root("[]");
    let app = workspace.path().join("my-app");
    fs::create_dir_all(&app).expect("create app dir");

    register_member(&app).expect("first registration");
    let after_first = members_of(workspace.path());
    let second = register_member(&app).expect("second registration");

    assert!(matches!(second, Registration::AlreadyCovered { .. }));
    assert_eq!(members_of(workspace.path()), after_first);
}

// --- glob coverage ---

#[test]
fn glob_covers_a_child_directory() {
    assert!(covers("crates/*", "crates/my-app"));
    assert!(!covers("crates/*", "apps/my-app"));
    assert!(!covers("crates/*", "crates/nested/my-app"));
}

#[test]
fn double_star_spans_segments() {
    assert!(covers("crates/**", "crates/nested/my-app"));
    assert!(covers("**", "anything/at/all"));
}

#[test]
fn exact_entries_match_themselves() {
    assert!(covers("crates/my-app", "crates/my-app"));
    assert!(!covers("crates/my-app", "crates/my-app-2"));
}

#[test]
fn a_trailing_slash_does_not_change_the_meaning() {
    assert!(covers("crates/*/", "crates/my-app"));
}

#[test]
fn partial_segment_globs_match() {
    assert!(covers("crates/nive-*", "crates/nive-ui"));
    assert!(!covers("crates/nive-*", "crates/other"));
    assert!(covers("*-app", "my-app"));
    assert!(!covers("a*a", "a"), "prefix and suffix must not overlap");
}

#[test]
fn exclude_reaches_everything_under_the_directory_it_names() {
    // `members` entries are globs; `exclude` also acts as a directory prefix.
    assert!(excludes("vendor", "vendor/my-app"));
    assert!(excludes("vendor/", "vendor/nested/my-app"));
    assert!(!covers("vendor", "vendor/my-app"));
    assert!(!excludes("vendor", "vendored/my-app"));
}

#[test]
fn a_glob_that_already_covers_the_crate_adds_no_entry() {
    let workspace = workspace_root("[\"crates/*\"]");
    fs::create_dir_all(workspace.path().join("crates")).expect("create crates dir");
    let before = members_of(workspace.path());

    scaffold(&workspace.path().join("crates"), "my-app");

    assert_eq!(
        members_of(workspace.path()),
        before,
        "a covering glob makes an explicit entry redundant"
    );
}

#[test]
fn an_excluded_path_is_left_standalone() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    fs::write(
        tempdir.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"keep\"]\nexclude = [\"vendor\"]\nresolver = \"2\"\n",
    )
    .expect("write workspace manifest");
    fs::create_dir_all(tempdir.path().join("vendor")).expect("create vendor dir");
    let before = members_of(tempdir.path());

    scaffold(&tempdir.path().join("vendor"), "my-app");

    assert_eq!(
        members_of(tempdir.path()),
        before,
        "an excluded path asked not to be a member"
    );
}

#[test]
fn without_a_workspace_nothing_outside_the_app_is_written() {
    let tempdir = tempfile::tempdir().expect("tempdir");

    scaffold(tempdir.path(), "my-app");

    let entries: Vec<_> = fs::read_dir(tempdir.path())
        .expect("read tempdir")
        .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "only the app directory should exist, got {entries:?}"
    );
}

#[test]
fn workspace_manifest_formatting_survives_registration() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let manifest = "\
# The crates that make up this product.
[workspace]
resolver = \"2\"
members = [
  # ordered by build dependency, not alphabetically
  \"core\",
  \"ui\",
]

[workspace.dependencies]
serde = \"1\"
";
    fs::write(tempdir.path().join("Cargo.toml"), manifest).expect("write workspace manifest");

    scaffold(tempdir.path(), "my-app");

    let updated = members_of(tempdir.path());
    assert!(
        updated.contains("# The crates that make up this product."),
        "leading comment lost:\n{updated}"
    );
    assert!(
        updated.contains("# ordered by build dependency, not alphabetically"),
        "in-array comment lost:\n{updated}"
    );
    assert!(
        updated.find("resolver").expect("resolver kept")
            < updated.find("members").expect("members kept"),
        "key order changed:\n{updated}"
    );
    assert!(
        updated.contains("\"my-app\""),
        "the new member is missing:\n{updated}"
    );
}

#[test]
fn a_workspace_without_a_members_key_gains_one() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    fs::write(
        tempdir.path().join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\n",
    )
    .expect("write workspace manifest");

    scaffold(tempdir.path(), "my-app");

    assert!(
        members_of(tempdir.path()).contains("\"my-app\""),
        "a workspace declared without members still claims the directories under it"
    );
}

#[test]
fn a_package_manifest_between_the_crate_and_the_root_is_walked_past() {
    let workspace = workspace_root("[\"host\"]");
    let host = workspace.path().join("host");
    fs::create_dir_all(&host).expect("create host dir");
    fs::write(
        host.join("Cargo.toml"),
        "[package]\nname = \"host\"\nversion = \"0.1.0\"\n",
    )
    .expect("write host manifest");

    scaffold(&host, "my-app");

    assert!(
        members_of(workspace.path()).contains("\"host/my-app\""),
        "a member crate in between is the ordinary nested layout, not a workspace:\n{}",
        members_of(workspace.path())
    );
    assert!(
        !fs::read_to_string(host.join("Cargo.toml"))
            .expect("read host manifest")
            .contains("workspace"),
        "the intermediate package manifest must not be edited"
    );
}

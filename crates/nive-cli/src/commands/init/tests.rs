use std::{fs, path::Path};

use super::*;
use crate::commands::icons;

/// A crate as `cargo new` leaves it: a manifest, and `src/main.rs`.
fn cargo_new(root: &Path, name: &str, main: Option<&str>) {
    fs::create_dir_all(root.join("src")).expect("create src");
    fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    )
    .expect("write manifest");

    if let Some(contents) = main {
        fs::write(root.join("src/main.rs"), contents).expect("write main.rs");
    }
}

fn adopt(root: &Path) -> Result<()> {
    run_in_dir(root, false, None, None, None, None)
}

#[test]
fn adopting_nive_in_an_empty_crate_produces_a_checkable_project() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    cargo_new(tempdir.path(), "my_app", None);

    adopt(tempdir.path()).expect("adopt nive");

    let manifest = fs::read_to_string(tempdir.path().join("Cargo.toml")).expect("read manifest");
    assert!(
        manifest.contains(r#"nive = "0.1""#),
        "the framework dependency is missing:\n{manifest}"
    );
    assert!(tempdir.path().join("src/main.rs").exists());
    assert!(tempdir.path().join("icons.toml").exists());

    icons::check_in_dir(tempdir.path()).expect("an adopted crate passes its icon check");
}

#[test]
fn the_generated_app_is_named_after_the_package() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    cargo_new(tempdir.path(), "my_app", None);

    adopt(tempdir.path()).expect("adopt nive");

    let main = fs::read_to_string(tempdir.path().join("src/main.rs")).expect("read main.rs");
    assert!(main.contains("struct MyApp"), "unexpected main.rs:\n{main}");
    assert!(main.contains(r#"ApplicationConfig::new("my_app")"#));
}

#[test]
fn authored_files_are_never_overwritten() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let authored = "fn main() { println!(\"mine\"); }\n";
    cargo_new(tempdir.path(), "my_app", Some(authored));

    adopt(tempdir.path()).expect("adopt nive");

    assert_eq!(
        fs::read_to_string(tempdir.path().join("src/main.rs")).expect("read main.rs"),
        authored,
        "adoption must not replace the author's code"
    );
    assert!(
        fs::read_to_string(tempdir.path().join("Cargo.toml"))
            .expect("read manifest")
            .contains("nive"),
        "the dependency is still established when files are skipped"
    );
}

#[test]
fn skipped_files_are_reported() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    cargo_new(tempdir.path(), "my_app", Some("fn main() {}\n"));

    let skipped =
        write_missing_templates(tempdir.path(), false, "my_app", "MyApp", r#"nive = "0.1""#)
            .expect("write missing templates");

    assert!(
        skipped.contains(&"src/main.rs".to_string()),
        "an existing file must be reported, not silently ignored: {skipped:?}"
    );
}

#[test]
fn adopting_twice_changes_nothing() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    cargo_new(tempdir.path(), "my_app", None);

    adopt(tempdir.path()).expect("first adoption");
    let after_first = snapshot(tempdir.path());

    adopt(tempdir.path()).expect("second adoption");

    assert_eq!(
        snapshot(tempdir.path()),
        after_first,
        "the second run must be a no-op"
    );
}

#[test]
fn an_existing_nive_dependency_is_left_alone() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(tempdir.path().join("src")).expect("create src");
    fs::write(
        tempdir.path().join("Cargo.toml"),
        "[package]\nname = \"my_app\"\nversion = \"0.1.0\"\n\n[dependencies]\nnive = { path = \"../nive\" }\n",
    )
    .expect("write manifest");

    adopt(tempdir.path()).expect("adopt nive");

    let manifest = fs::read_to_string(tempdir.path().join("Cargo.toml")).expect("read manifest");
    assert!(
        manifest.contains(r#"path = "../nive""#),
        "an author's own dependency source must survive:\n{manifest}"
    );
}

#[test]
fn adoption_outside_a_package_fails_without_writing() {
    let tempdir = tempfile::tempdir().expect("tempdir");

    let error = adopt(tempdir.path()).expect_err("no package to adopt into");

    assert!(
        error.to_string().contains("nive new"),
        "the error should point at the command that creates a crate: {error}"
    );
    assert_eq!(
        fs::read_dir(tempdir.path()).expect("read tempdir").count(),
        0,
        "nothing may be written when the command refuses"
    );
}

#[test]
fn a_virtual_manifest_is_not_a_package() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    fs::write(
        tempdir.path().join("Cargo.toml"),
        "[workspace]\nmembers = []\n",
    )
    .expect("write manifest");

    let error = adopt(tempdir.path()).expect_err("a workspace root is not a package");

    assert!(
        error.to_string().contains("[package]"),
        "unexpected error: {error}"
    );
}

/// Every file under `root`, with its contents, for comparing two runs.
fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut entries = Vec::new();
    collect(root, root, &mut entries);
    entries.sort();
    entries
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    for entry in fs::read_dir(dir).expect("read dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, out);
        } else {
            let relative = path
                .strip_prefix(root)
                .expect("under root")
                .to_string_lossy()
                .into_owned();
            out.push((relative, fs::read(&path).expect("read file")));
        }
    }
}

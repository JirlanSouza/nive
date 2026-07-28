//! Adopting Nive in a Cargo package that already exists.
//!
//! The rule that makes this safe to run in a directory with real work in it: it
//! writes only files that do not exist, and reports the ones it left alone.

use std::{fs, path::Path};

use toml_edit::{DocumentMut, Item, Table};

use super::icons;
use super::new::{
    build_nive_dep, for_each_template, render_template, target_relative_path, templates_for,
    to_title_case,
};
use super::workspace::register_member;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod tests;

/// Files `nive icons init` owns; it knows what the app's manifest declares and
/// the template does not.
const ICON_OWNED: &[&str] = &[
    "icons.toml",
    "src/icons.rs",
    "src/icons/generated.rs",
    "src/icons/generated/catalog.rs",
    "src/icons/generated/symbols.rs",
];

pub fn run(
    dashboard: bool,
    git: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
    branch: Option<&str>,
) -> Result<()> {
    let root = std::env::current_dir()?;
    run_in_dir(&root, dashboard, git, tag, rev, branch)
}

pub(super) fn run_in_dir(
    root: &Path,
    dashboard: bool,
    git: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
    branch: Option<&str>,
) -> Result<()> {
    let nive_dep = build_nive_dep(git, tag, rev, branch, dashboard)?;
    let manifest_path = root.join("Cargo.toml");
    let package_name = read_package_name(&manifest_path)?;

    println!(
        "Adding Nive to {} ({})",
        package_name,
        if dashboard { "dashboard" } else { "basic" }
    );

    add_nive_dependency(&manifest_path, &nive_dep)?;
    icons::init_in_dir(root)?;

    let title = to_title_case(&package_name);
    let skipped = write_missing_templates(root, dashboard, &package_name, &title, &nive_dep)?;

    if let Some(report) = register_member(root)?.report() {
        println!("{report}");
    }

    println!("\nSuccess! {} now depends on Nive.", package_name);

    if !skipped.is_empty() {
        println!("\nLeft untouched, because these already exist:");
        for path in &skipped {
            println!("  {path}");
        }
        println!(
            "\nCompare them with `nive new {}` in a scratch directory if you want the template's version.",
            package_name
        );
    }

    println!("\nNext steps:");
    println!("  cargo build");
    println!("  nive icons check");

    Ok(())
}

fn read_package_name(manifest_path: &Path) -> Result<String> {
    if !manifest_path.exists() {
        return Err(format!(
            "No `Cargo.toml` at {}. `nive init` adds Nive to an existing crate; use `nive new <name>` to create one.",
            manifest_path.display()
        )
        .into());
    }

    let document = fs::read_to_string(manifest_path)?.parse::<DocumentMut>()?;

    document
        .get("package")
        .and_then(Item::as_table)
        .and_then(|package| package.get("name"))
        .and_then(Item::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "{} declares no `[package]`. `nive init` adds Nive to a crate; run it inside the package directory, not at a workspace root.",
                manifest_path.display()
            )
            .into()
        })
}

/// The dependency line is parsed back from `build_nive_dep` rather than rebuilt
/// here, so `new` and `init` cannot disagree about what `--git` and `--tag`
/// produce.
fn add_nive_dependency(manifest_path: &Path, nive_dep: &str) -> Result<()> {
    let mut document = fs::read_to_string(manifest_path)?.parse::<DocumentMut>()?;

    if document
        .get("dependencies")
        .and_then(Item::as_table)
        .is_some_and(|dependencies| dependencies.contains_key("nive"))
    {
        println!("  `nive` is already a dependency; leaving it as it is");
        return Ok(());
    }

    let parsed = nive_dep.parse::<DocumentMut>()?;
    let value = parsed
        .get("nive")
        .cloned()
        .expect("build_nive_dep always emits a `nive` key");

    if !document.contains_key("dependencies") {
        document["dependencies"] = Item::Table(Table::new());
    }

    document["dependencies"]["nive"] = value;
    fs::write(manifest_path, document.to_string())?;
    println!("  Added `nive` to {}", manifest_path.display());

    Ok(())
}

/// Returns the paths it refused to overwrite, for reporting.
fn write_missing_templates(
    root: &Path,
    dashboard: bool,
    app_name: &str,
    title: &str,
    nive_dep: &str,
) -> Result<Vec<String>> {
    let mut skipped = Vec::new();

    for_each_template(templates_for(dashboard), &mut |file| {
        let relative = target_relative_path(file);

        // Handled by `add_nive_dependency` and `icons::init_in_dir`, which edit
        // rather than replace.
        if relative == "Cargo.toml" || ICON_OWNED.contains(&relative.as_str()) {
            return Ok(());
        }

        let target = root.join(&relative);
        if target.exists() {
            skipped.push(relative);
            return Ok(());
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&target, render_template(file, app_name, title, nive_dep))?;
        println!("  Created {relative}");

        Ok(())
    })?;

    Ok(skipped)
}

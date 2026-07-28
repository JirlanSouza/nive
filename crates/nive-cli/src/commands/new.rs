use std::{fs, path::Path};

use include_dir::{include_dir, Dir};

use super::workspace::register_member;

static BASIC_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/basic");
static DASHBOARD_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/dashboard");

pub(super) fn to_title_case(s: &str) -> String {
    s.split(['_', '-'])
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    upper + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

/// Builds the TOML value for the `nive` dependency line given the source args.
///
/// - No git args → crates.io stable form
/// - `--git <url>` plus exactly one of `--tag`, `--rev`, or `--branch` → Git form
///
/// Returns an error for invalid combinations.
pub(super) fn build_nive_dep(
    git: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
    branch: Option<&str>,
    dashboard: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let ref_count = [tag, rev, branch].iter().filter(|o| o.is_some()).count();

    if git.is_none() {
        if ref_count > 0 {
            return Err("--tag, --rev, and --branch require --git".into());
        }
        if dashboard {
            return Ok(r#"nive = { version = "0.1", features = ["file-picker"] }"#.to_string());
        }
        return Ok(r#"nive = "0.1""#.to_string());
    }

    let url = git.unwrap();

    if ref_count == 0 {
        return Err("--git requires exactly one of --tag, --rev, or --branch".into());
    }
    if ref_count > 1 {
        return Err("--git requires exactly one of --tag, --rev, or --branch; got multiple".into());
    }

    let ref_part = if let Some(t) = tag {
        format!(r#", tag = "{}""#, t)
    } else if let Some(r) = rev {
        format!(r#", rev = "{}""#, r)
    } else {
        format!(r#", branch = "{}""#, branch.unwrap())
    };

    if dashboard {
        Ok(format!(
            r#"nive = {{ git = "{}"{}, features = ["file-picker"] }}"#,
            url, ref_part
        ))
    } else {
        Ok(format!(r#"nive = {{ git = "{}"{} }}"#, url, ref_part))
    }
}

pub(super) fn templates_for(dashboard: bool) -> &'static Dir<'static> {
    if dashboard {
        &DASHBOARD_TEMPLATES
    } else {
        &BASIC_TEMPLATES
    }
}

pub(super) fn target_relative_path(file: &include_dir::File) -> String {
    file.path().to_string_lossy().replace(".template", "")
}

pub(super) fn render_template(
    file: &include_dir::File,
    app_name: &str,
    title: &str,
    nive_dep: &str,
) -> String {
    file.contents_utf8()
        .unwrap_or("")
        .replace("{{app_name}}", app_name)
        .replace("{{app_name_title}}", title)
        .replace("{{nive_dep}}", nive_dep)
}

pub(super) fn for_each_template(
    dir: &'static Dir<'static>,
    visit: &mut dyn FnMut(
        &'static include_dir::File<'static>,
    ) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    for file in dir.files() {
        visit(file)?;
    }

    for subdir in dir.dirs() {
        for_each_template(subdir, visit)?;
    }

    Ok(())
}

fn copy_templates(
    dir: &'static Dir<'static>,
    app_dir: &Path,
    app_name: &str,
    title: &str,
    nive_dep: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for_each_template(dir, &mut |file| {
        let target_relative = target_relative_path(file);
        let target_path = app_dir.join(&target_relative);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(
            &target_path,
            render_template(file, app_name, title, nive_dep),
        )?;
        println!("  Created {}", target_relative);

        Ok(())
    })
}

pub fn run(
    name: &str,
    dashboard: bool,
    git: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
    branch: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = std::env::current_dir()?;
    run_in_dir(&parent, name, dashboard, git, tag, rev, branch)
}

pub(super) fn run_in_dir(
    parent: &Path,
    name: &str,
    dashboard: bool,
    git: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
    branch: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = &parent.join(name);

    if app_dir.exists() {
        return Err(format!("Directory already exists: {}", app_dir.display()).into());
    }

    let nive_dep = build_nive_dep(git, tag, rev, branch, dashboard)?;

    let title = to_title_case(name);
    let templates = templates_for(dashboard);

    println!(
        "Creating new Nive app: {} ({})",
        name,
        if dashboard { "dashboard" } else { "basic" }
    );

    fs::create_dir_all(app_dir)?;

    copy_templates(templates, app_dir, name, &title, &nive_dep)?;

    if let Some(report) = register_member(app_dir)?.report() {
        println!("{report}");
    }

    println!("\nSuccess! Created {} at {}", name, app_dir.display());
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  cargo build");
    println!("  just dev");

    Ok(())
}

#[cfg(test)]
mod tests;

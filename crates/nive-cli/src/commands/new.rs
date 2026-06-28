use std::fs;
use std::path::Path;

use include_dir::{include_dir, Dir};

static BASIC_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/basic");
static DASHBOARD_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/dashboard");

fn to_title_case(s: &str) -> String {
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
fn build_nive_dep(
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

fn copy_templates(
    dir: &Dir,
    app_dir: &Path,
    app_name: &str,
    title: &str,
    nive_dep: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for file in dir.files() {
        let relative_path = file.path();
        let target_relative = relative_path.to_string_lossy().replace(".template", "");
        let target_path = app_dir.join(&target_relative);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = file.contents_utf8().unwrap_or("");
        let content = content.replace("{{app_name}}", app_name);
        let content = content.replace("{{app_name_title}}", title);
        let content = content.replace("{{nive_dep}}", nive_dep);

        fs::write(&target_path, content)?;
        println!("  Created {}", target_relative);
    }

    for subdir in dir.dirs() {
        copy_templates(subdir, app_dir, app_name, title, nive_dep)?;
    }

    Ok(())
}

pub fn run(
    name: &str,
    dashboard: bool,
    git: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
    branch: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = Path::new(name);

    if app_dir.exists() {
        return Err(format!("Directory already exists: {}", app_dir.display()).into());
    }

    let nive_dep = build_nive_dep(git, tag, rev, branch, dashboard)?;

    let title = to_title_case(name);
    let templates = if dashboard {
        &DASHBOARD_TEMPLATES
    } else {
        &BASIC_TEMPLATES
    };

    println!(
        "Creating new Nive app: {} ({})",
        name,
        if dashboard { "dashboard" } else { "basic" }
    );

    fs::create_dir_all(app_dir)?;

    copy_templates(templates, app_dir, name, &title, &nive_dep)?;

    println!("\nSuccess! Created {} at {}", name, app_dir.display());
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  cargo build");
    println!("  just dev");

    Ok(())
}

#[cfg(test)]
mod tests {
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
}

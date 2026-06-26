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

fn copy_templates(
    dir: &Dir,
    app_dir: &Path,
    app_name: &str,
    title: &str,
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

        fs::write(&target_path, content)?;
        println!("  Created {}", target_relative);
    }

    for subdir in dir.dirs() {
        copy_templates(subdir, app_dir, app_name, title)?;
    }

    Ok(())
}

pub fn run(name: &str, dashboard: bool) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = Path::new(name);

    if app_dir.exists() {
        eprintln!("Error: Directory already exists: {}", app_dir.display());
        std::process::exit(1);
    }

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

    copy_templates(templates, app_dir, name, &title)?;

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

        copy_templates(&BASIC_TEMPLATES, &app_dir, "my_app", "MyApp").expect("copy");

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
    }

    #[test]
    fn copy_templates_writes_all_basic_template_files() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let app_dir = tempdir.path().join("basic-app");

        copy_templates(&BASIC_TEMPLATES, &app_dir, "basic_app", "BasicApp").expect("copy");

        let expected = collect_relative_files(&BASIC_TEMPLATES)
            .into_iter()
            .map(|p| p.replace(".template", ""))
            .collect::<Vec<_>>();
        for rel in expected {
            let path = app_dir.join(&rel);
            assert!(path.exists(), "missing {} after copy", rel);
        }
    }
}

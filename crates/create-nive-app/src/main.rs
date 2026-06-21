use std::fs;
use std::path::PathBuf;

use clap::Parser;
use include_dir::{include_dir, Dir};

static TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/basic");

#[derive(Parser)]
#[command(name = "create-nive-app")]
#[command(about = "Create a new Nive app")]
struct Cli {
    name: String,

    #[arg(short, long, default_value = ".")]
    path: PathBuf,
}

fn to_title_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-')
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
    app_dir: &std::path::Path,
    app_name: &str,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for file in dir.files() {
        let relative_path = file.path();
        let target_relative = relative_path
            .to_string_lossy()
            .replace(".template", "");
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let app_dir = cli.path.join(&cli.name);

    if app_dir.exists() {
        eprintln!("Error: Directory already exists: {}", app_dir.display());
        std::process::exit(1);
    }

    let title = to_title_case(&cli.name);

    println!("Creating new Nive app: {}", cli.name);

    fs::create_dir_all(&app_dir)?;

    copy_templates(&TEMPLATES, &app_dir, &cli.name, &title)?;

    println!("\nSuccess! Created {} at {}", cli.name, app_dir.display());
    println!("\nNext steps:");
    println!("  cd {}", app_dir.display());
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
}

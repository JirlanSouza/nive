use std::{
    ffi::OsStr,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use flate2::read::GzDecoder;
use serde::{Deserialize, Deserializer};
use tar::Archive;

use super::{
    IconPaths, LucideIconMetadata, LucideIconMetadataFile, LucideMetadata, LucideProvider, Result,
};

pub(super) fn fetch_lucide_svg(version: &str, slug: &str) -> Result<String> {
    let url =
        format!("https://raw.githubusercontent.com/lucide-icons/lucide/{version}/icons/{slug}.svg");

    fetch_text(&url)
}

impl LucideMetadata {
    const CACHE_VERSION: u8 = 1;

    pub(super) fn search(&self, query: &str) -> Vec<&LucideIconMetadata> {
        self.icons
            .iter()
            .filter(|icon| icon.matches(query))
            .collect()
    }

    pub(super) fn list(&self, category: Option<&str>) -> Vec<&LucideIconMetadata> {
        self.icons
            .iter()
            .filter(|icon| {
                category.is_none_or(|category| {
                    icon.categories
                        .iter()
                        .any(|candidate| candidate.eq_ignore_ascii_case(category))
                })
            })
            .collect()
    }

    pub(super) fn find(&self, slug: &str) -> Option<&LucideIconMetadata> {
        self.icons.iter().find(|icon| icon.name == slug)
    }

    /// Offline provider metadata: enough icons to exercise alias, tag, category,
    /// and use-case matching. A fixture, not a second opinion about what Lucide
    /// or `nive-ui` contains.
    #[cfg(test)]
    pub(super) fn fallback() -> Self {
        let mut icons = vec![
            LucideIconMetadata {
                name: "user".to_string(),
                aliases: vec!["account".to_string(), "person".to_string()],
                tags: vec!["profile".to_string(), "avatar".to_string()],
                categories: vec!["users".to_string()],
                use_cases: vec!["account menu".to_string()],
            },
            LucideIconMetadata {
                name: "arrow-up".to_string(),
                aliases: vec!["up".to_string()],
                tags: vec!["north".to_string(), "direction".to_string()],
                categories: vec!["arrows".to_string(), "navigation".to_string()],
                use_cases: vec!["sort ascending".to_string()],
            },
            LucideIconMetadata {
                name: "shield-check".to_string(),
                aliases: vec!["security".to_string()],
                tags: vec!["safe".to_string(), "verified".to_string()],
                categories: vec!["security".to_string()],
                use_cases: vec!["verified state".to_string()],
            },
            LucideIconMetadata {
                name: "x".to_string(),
                aliases: vec!["close".to_string()],
                tags: vec!["dismiss".to_string()],
                categories: vec!["navigation".to_string()],
                use_cases: vec!["close a window".to_string()],
            },
        ];

        icons.sort_by(|left, right| left.name.cmp(&right.name));
        Self {
            cache_version: Self::CACHE_VERSION,
            icons,
        }
    }
}

impl LucideIconMetadata {
    pub(super) fn matches(&self, query: &str) -> bool {
        self.name.contains(query)
            || self.aliases.iter().any(|value| contains(value, query))
            || self.tags.iter().any(|value| contains(value, query))
            || self.categories.iter().any(|value| contains(value, query))
            || self.use_cases.iter().any(|value| contains(value, query))
    }

    pub(super) fn summary_line(&self) -> String {
        let mut parts = vec![format!("lucide:{}", self.name)];

        if !self.categories.is_empty() {
            parts.push(format!("categories={}", self.categories.join(",")));
        }
        if !self.tags.is_empty() {
            parts.push(format!("tags={}", self.tags.join(",")));
        }
        if !self.aliases.is_empty() {
            parts.push(format!("aliases={}", self.aliases.join(",")));
        }

        parts.join("  ")
    }
}

impl<'a> LucideProvider<'a> {
    pub(super) fn new(paths: &'a IconPaths, version: &'a str) -> Self {
        Self { paths, version }
    }

    pub(super) fn metadata(&self) -> Result<LucideMetadata> {
        fs::create_dir_all(&self.paths.metadata_cache_dir)?;
        let cache_path = self
            .paths
            .metadata_cache_dir
            .join(format!("lucide-{}.json", self.version));

        if cache_path.exists() {
            let contents = fs::read_to_string(&cache_path)?;
            let metadata: LucideMetadata = serde_json::from_str(&contents)?;
            if metadata.cache_version == LucideMetadata::CACHE_VERSION {
                return Ok(metadata);
            }
        }

        let metadata = self.fetch_metadata()?;
        fs::write(&cache_path, serde_json::to_string_pretty(&metadata)?)?;
        Ok(metadata)
    }

    pub(super) fn fetch_metadata(&self) -> Result<LucideMetadata> {
        let url = format!(
            "https://codeload.github.com/lucide-icons/lucide/tar.gz/{}",
            self.version
        );
        let archive_bytes = fetch_bytes(&url)?;
        let decoder = GzDecoder::new(archive_bytes.as_slice());
        let mut archive = Archive::new(decoder);
        let mut icons = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let Some(slug) = lucide_metadata_slug(&path) else {
                continue;
            };

            let mut contents = String::new();
            entry.read_to_string(&mut contents)?;
            let file: LucideIconMetadataFile =
                serde_json::from_str(&contents).map_err(|error| {
                    format!("Failed to parse Lucide metadata for `{slug}` from {url}: {error}")
                })?;
            icons.push(LucideIconMetadata {
                name: slug,
                aliases: file.aliases,
                tags: file.tags,
                categories: file.categories,
                use_cases: file.use_cases,
            });
        }

        icons.sort_by(|left, right| left.name.cmp(&right.name));
        icons.dedup_by(|left, right| left.name == right.name);
        if icons.is_empty() {
            return Err(format!(
                "Lucide metadata archive for {} contained no icons.",
                self.version
            )
            .into());
        }

        Ok(LucideMetadata {
            cache_version: LucideMetadata::CACHE_VERSION,
            icons,
        })
    }
}

pub(super) fn fetch_text(url: &str) -> Result<String> {
    let mut response = ureq::get(url)
        .header("User-Agent", "nive-cli")
        .call()
        .map_err(|error| format!("Failed to fetch {url}: {error}"))?;

    response
        .body_mut()
        .read_to_string()
        .map_err(|error| format!("Failed to read response for {url}: {error}").into())
}

pub(super) fn fetch_bytes(url: &str) -> Result<Vec<u8>> {
    let mut response = ureq::get(url)
        .header("User-Agent", "nive-cli")
        .call()
        .map_err(|error| format!("Failed to fetch {url}: {error}"))?;

    response
        .body_mut()
        .read_to_vec()
        .map_err(|error| format!("Failed to read response for {url}: {error}").into())
}

pub(super) fn lucide_metadata_slug(path: &Path) -> Option<String> {
    let mut components = path.components();
    components.next()?;

    if components.next()?.as_os_str() != OsStr::new("icons") {
        return None;
    }

    let file_name = components.next()?.as_os_str().to_str()?;
    if components.next().is_some() {
        return None;
    }

    file_name.strip_suffix(".json").map(str::to_string)
}

pub(super) fn deserialize_string_values<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let mut values = Vec::new();
    collect_string_values(&value, &mut values);
    Ok(values)
}

pub(super) fn collect_string_values(value: &serde_json::Value, values: &mut Vec<String>) {
    match value {
        serde_json::Value::String(value) => values.push(value.clone()),
        serde_json::Value::Array(items) => {
            for item in items {
                collect_string_values(item, values);
            }
        }
        serde_json::Value::Object(fields) => {
            if let Some(serde_json::Value::String(name)) = fields.get("name") {
                values.push(name.clone());
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

pub(super) fn contains(value: &str, query: &str) -> bool {
    value.to_ascii_lowercase().contains(query)
}

pub(super) fn print_metadata_list(label: &str, values: &[String]) {
    if !values.is_empty() {
        println!("{label}: {}", values.join(", "));
    }
}

pub(super) fn generate_gallery_html(metadata: &LucideMetadata) -> String {
    let mut html = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Lucide Icon Gallery</title>\
         <style>body{font:14px system-ui,sans-serif;margin:24px;color:#17202a}\
         .grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:12px}\
         .item{border:1px solid #d7dde5;border-radius:6px;padding:12px}\
         code{font-size:12px;color:#425466}</style></head><body>\
         <h1>Lucide Icon Gallery</h1><div class=\"grid\">",
    );

    for icon in &metadata.icons {
        html.push_str("<div class=\"item\"><strong>");
        html.push_str(&escape_html(&icon.name));
        html.push_str("</strong><br><code>lucide:");
        html.push_str(&escape_html(&icon.name));
        html.push_str("</code>");
        if !icon.categories.is_empty() {
            html.push_str("<br>");
            html.push_str(&escape_html(&icon.categories.join(", ")));
        }
        html.push_str("</div>");
    }

    html.push_str("</div></body></html>\n");
    html
}

pub(super) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(super) fn open_path(path: &Path) -> Result<()> {
    let status = if cfg!(target_os = "macos") {
        Command::new("open").arg(path).status()?
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg(path)
            .status()?
    } else {
        Command::new("xdg-open").arg(path).status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!("Failed to open {}", display_path(path)).into())
    }
}

pub(super) fn display_path(path: &Path) -> String {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.strip_prefix(&cwd)
        .unwrap_or(path)
        .display()
        .to_string()
}

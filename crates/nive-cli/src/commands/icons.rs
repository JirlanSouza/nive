use std::{collections::BTreeMap, path::PathBuf};

use clap::Subcommand;

mod commands;
mod generate;
mod lucide;
mod manifest;

#[cfg(test)]
mod tests;

use self::commands::run_in_dir;
use self::lucide::deserialize_string_values;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct IconsManifest {
    #[serde(default)]
    provider: ProviderConfig,
    #[serde(default)]
    roles: BTreeMap<String, String>,
    #[serde(default)]
    symbols: BTreeMap<String, String>,
    #[serde(default)]
    custom: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProviderConfig {
    #[serde(default)]
    lucide: LucideProviderConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct LucideProviderConfig {
    version: String,
    stroke_width: String,
    stroke_linecap: String,
    stroke_linejoin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ProviderRef {
    Lucide(String),
    Custom(String),
}

struct IconPaths {
    root: PathBuf,
    manifest: PathBuf,
    generated_asset_dir: PathBuf,
    icons_rs: PathBuf,
    generated_rs: PathBuf,
    generated_catalog: PathBuf,
    generated_symbols: PathBuf,
    gallery_dir: PathBuf,
    metadata_cache_dir: PathBuf,
    legacy_manifest: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IconGenerationTarget {
    App,
    Framework,
}

#[derive(Subcommand)]
pub enum IconsCommands {
    /// List icons declared in the current manifest, or list provider metadata with --provider
    List {
        /// Provider to list from discovery metadata
        #[arg(long)]
        provider: Option<String>,
        /// Provider category to filter
        #[arg(long)]
        category: Option<String>,
    },
    /// Fetch declared SVGs and regenerate named icon Rust modules
    Sync {
        /// Generate framework-internal modules for the nive-ui crate
        #[arg(long, default_value_t = false)]
        framework: bool,
    },
    /// Check if manifest, generated assets, and generated modules are in sync without network
    Check {
        /// Check framework-internal modules for the nive-ui crate
        #[arg(long, default_value_t = false)]
        framework: bool,
    },
    /// Add an app-owned symbol to icons.toml and sync
    #[command(name = "add-symbol")]
    AddSymbol {
        /// PascalCase symbol variant (e.g., User)
        variant: String,
        /// Provider ref, e.g. lucide:user or custom:brand-mark. Bare values are Lucide shorthand.
        provider_ref: String,
        /// Write framework-internal modules for the nive-ui crate
        #[arg(long, default_value_t = false)]
        framework: bool,
    },
    /// Map a semantic role to a provider ref and sync
    #[command(name = "set-role")]
    SetRole {
        /// Canonical role name, e.g. window-close
        role_name: String,
        /// Provider ref, e.g. lucide:x or custom:brand-close. Bare values are Lucide shorthand.
        provider_ref: String,
        /// Write framework-internal modules for the nive-ui crate
        #[arg(long, default_value_t = false)]
        framework: bool,
    },
    /// Register a custom SVG in icons.toml
    #[command(name = "add-custom")]
    AddCustom {
        /// Custom icon name used by `custom:<name>` refs
        name: String,
        /// Manifest-relative SVG path
        path: String,
    },
    /// Scaffold icons.toml and named generated modules in the current app
    Init,
    /// Search Lucide provider metadata
    Search {
        query: String,
        #[arg(long, default_value = "lucide")]
        provider: String,
    },
    /// Show provider metadata for one icon ref
    Show { provider_ref: String },
    /// Generate a local provider gallery HTML file
    Gallery {
        #[arg(long, default_value = "lucide")]
        provider: String,
        #[arg(long, default_value_t = false)]
        open: bool,
    },
    /// Deprecated. Use add-symbol instead.
    Add {
        variant: String,
        provider_ref: String,
    },
}

pub fn run(command: IconsCommands) -> Result<()> {
    let cwd = std::env::current_dir()?;
    run_in_dir(command, &cwd)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LucideMetadata {
    #[serde(default)]
    cache_version: u8,
    icons: Vec<LucideIconMetadata>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LucideIconMetadata {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default, rename = "useCases")]
    use_cases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LucideIconMetadataFile {
    #[serde(default, deserialize_with = "deserialize_string_values")]
    aliases: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_values")]
    tags: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_values")]
    categories: Vec<String>,
    #[serde(
        default,
        rename = "useCases",
        deserialize_with = "deserialize_string_values"
    )]
    use_cases: Vec<String>,
}

struct LucideProvider<'a> {
    paths: &'a IconPaths,
    version: &'a str,
}

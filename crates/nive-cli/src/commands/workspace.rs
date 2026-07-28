//! Registering a generated crate with the Cargo workspace that encloses it.
//!
//! Cargo refuses to build a package that sits under a workspace root without
//! being one of its members, so a scaffold that writes only its own directory
//! hands the author a project that does not compile.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use toml_edit::{Array, DocumentMut, Item, Table, Value};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// What happened when a crate met the workspace above it. Every variant except
/// `Registered` means the build already works, which is all registration is for.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Registration {
    Registered { manifest: PathBuf, member: String },
    AlreadyCovered { manifest: PathBuf, member: String },
    Excluded { manifest: PathBuf, member: String },
    NoWorkspace,
}

impl Registration {
    pub(super) fn report(&self) -> Option<String> {
        match self {
            Self::Registered { manifest, member } => {
                Some(format!("  Registered `{member}` in {}", manifest.display()))
            }
            Self::AlreadyCovered { manifest, member } => Some(format!(
                "  `{member}` is already a member of {}",
                manifest.display()
            )),
            Self::Excluded { manifest, member } => Some(format!(
                "  `{member}` is excluded by {}; leaving it standalone",
                manifest.display()
            )),
            Self::NoWorkspace => None,
        }
    }
}

/// `crate_dir` must already exist, so both ends can be canonicalized and no `..`
/// reaches the manifest.
pub(super) fn register_member(crate_dir: &Path) -> Result<Registration> {
    let crate_dir = crate_dir.canonicalize()?;

    let Some(manifest_path) = find_workspace_manifest(&crate_dir) else {
        return Ok(Registration::NoWorkspace);
    };

    let root = manifest_path
        .parent()
        .expect("a manifest path always has a parent directory");
    let Some(member) = relative_member_path(root, &crate_dir) else {
        return Ok(Registration::NoWorkspace);
    };

    let mut document = fs::read_to_string(&manifest_path)?.parse::<DocumentMut>()?;
    let workspace = document
        .get("workspace")
        .and_then(Item::as_table)
        .expect("find_workspace_manifest only returns manifests with [workspace]");

    if list_entries(workspace, "exclude")
        .iter()
        .any(|pattern| excludes(pattern, &member))
    {
        return Ok(Registration::Excluded {
            manifest: manifest_path,
            member,
        });
    }

    if list_entries(workspace, "members")
        .iter()
        .any(|pattern| covers(pattern, &member))
    {
        return Ok(Registration::AlreadyCovered {
            manifest: manifest_path,
            member,
        });
    }

    append_member(&mut document, &member);
    fs::write(&manifest_path, document.to_string())?;

    Ok(Registration::Registered {
        manifest: manifest_path,
        member,
    })
}

/// The nearest ancestor manifest declaring a workspace, walking past ancestors
/// that declare only a package: a member crate in between is the ordinary
/// nested layout, not a stopping point.
fn find_workspace_manifest(crate_dir: &Path) -> Option<PathBuf> {
    crate_dir
        .ancestors()
        .skip(1)
        .map(|dir| dir.join("Cargo.toml"))
        .find(|manifest| {
            fs::read_to_string(manifest)
                .ok()
                .and_then(|source| source.parse::<DocumentMut>().ok())
                .is_some_and(|document| document.contains_key("workspace"))
        })
}

fn relative_member_path(root: &Path, crate_dir: &Path) -> Option<String> {
    let relative = crate_dir.strip_prefix(root).ok()?;
    let segments: Vec<_> = relative
        .components()
        .map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Option<_>>()?;

    (!segments.is_empty()).then(|| segments.join("/"))
}

fn list_entries(table: &Table, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(Item::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn covers(pattern: &str, path: &str) -> bool {
    let pattern: Vec<&str> = pattern.trim_end_matches('/').split('/').collect();
    let path: Vec<&str> = path.split('/').collect();

    matches_segments(&pattern, &path)
}

/// Broader than [`covers`]: an `exclude` entry acts as a directory prefix as
/// well as a glob, so `vendor` keeps `vendor/my-app` out. Reading it as a plain
/// glob would register a crate the author deliberately excluded.
fn excludes(pattern: &str, path: &str) -> bool {
    let prefix = pattern.trim_end_matches('/');

    covers(pattern, path)
        || path
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn matches_segments(pattern: &[&str], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((&"**", rest)) => (0..=path.len()).any(|skip| matches_segments(rest, &path[skip..])),
        Some((head, rest)) => match path.split_first() {
            Some((segment, tail)) if matches_segment(head, segment) => matches_segments(rest, tail),
            _ => false,
        },
    }
}

fn matches_segment(pattern: &str, segment: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    let (Some(prefix), Some(suffix)) = (parts.first(), parts.last()) else {
        return pattern == segment;
    };

    if parts.len() == 1 {
        return pattern == segment;
    }

    let Some(mut rest) = segment.strip_prefix(prefix) else {
        return false;
    };

    for part in &parts[1..parts.len() - 1] {
        match rest.find(part) {
            Some(at) => rest = &rest[at + part.len()..],
            None => return false,
        }
    }

    // Without the length guard a prefix and a suffix claim the same bytes, and
    // `a*a` matches `a`.
    rest.len() >= suffix.len() && rest.ends_with(suffix)
}

/// Formatting is copied from the last entry so the addition sits in whatever
/// layout the author was already using, one-per-line or inline.
fn append_member(document: &mut DocumentMut, member: &str) {
    let workspace = document["workspace"]
        .as_table_mut()
        .expect("checked before this point");

    if !workspace.contains_key("members") {
        workspace["members"] = Item::Value(Value::Array(Array::new()));
    }

    let members = workspace["members"]
        .as_array_mut()
        .expect("members is an array");

    let decor = members.iter().last().map(|last| last.decor().clone());
    members.push(member);

    if let (Some(decor), Some(added)) = (decor, members.iter_mut().last()) {
        *added.decor_mut() = decor;
    }
}

#[cfg(test)]
mod tests;

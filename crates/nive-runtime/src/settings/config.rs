use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsConfig {
    path: PathBuf,
}

impl SettingsConfig {
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

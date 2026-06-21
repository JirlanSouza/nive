use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{RuntimeSession, SettingsConfig};

const SESSION_FILE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsError {
    kind: SettingsErrorKind,
    path: PathBuf,
    detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsErrorKind {
    Read,
    Write,
    Parse,
    UnsupportedVersion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RuntimeSessionFile {
    version: u32,
    session: RuntimeSession,
}

pub(crate) fn load_session(
    config: &SettingsConfig,
) -> Result<Option<RuntimeSession>, SettingsError> {
    load_session_from_path(config.path())
}

pub(crate) fn save_session(
    config: &SettingsConfig,
    session: &RuntimeSession,
) -> Result<(), SettingsError> {
    save_session_to_path(config.path(), session)
}

fn load_session_from_path(path: &Path) -> Result<Option<RuntimeSession>, SettingsError> {
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(SettingsError::new(
                SettingsErrorKind::Read,
                path,
                error.to_string(),
            ));
        }
    };

    let file = serde_json::from_str::<RuntimeSessionFile>(&contents)
        .map_err(|error| SettingsError::new(SettingsErrorKind::Parse, path, error.to_string()))?;

    if file.version != SESSION_FILE_VERSION {
        return Err(SettingsError::new(
            SettingsErrorKind::UnsupportedVersion,
            path,
            file.version.to_string(),
        ));
    }

    Ok(Some(file.session))
}

fn save_session_to_path(path: &Path, session: &RuntimeSession) -> Result<(), SettingsError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            SettingsError::new(SettingsErrorKind::Write, path, error.to_string())
        })?;
    }

    let file = RuntimeSessionFile {
        version: SESSION_FILE_VERSION,
        session: session.clone(),
    };
    let contents = serde_json::to_string_pretty(&file)
        .map_err(|error| SettingsError::new(SettingsErrorKind::Write, path, error.to_string()))?;

    std::fs::write(path, contents)
        .map_err(|error| SettingsError::new(SettingsErrorKind::Write, path, error.to_string()))
}

impl SettingsError {
    fn new(kind: SettingsErrorKind, path: &Path, detail: String) -> Self {
        Self {
            kind,
            path: path.to_path_buf(),
            detail,
        }
    }

    pub fn kind(&self) -> SettingsErrorKind {
        self.kind
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn detail(&self) -> &str {
        self.detail.as_str()
    }
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "settings {:?} error at {}: {}",
            self.kind,
            self.path.display(),
            self.detail
        )
    }
}

impl std::error::Error for SettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThemePreference;

    #[test]
    fn missing_session_file_falls_back_to_config_defaults() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("missing.json");

        assert_eq!(load_session_from_path(&path), Ok(None));
    }

    #[test]
    fn corrupt_session_file_falls_back_to_config_defaults() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("settings.json");
        std::fs::write(&path, "not-json").expect("write corrupt settings");

        let error = load_session_from_path(&path).expect_err("corrupt file should error");

        assert_eq!(error.kind(), SettingsErrorKind::Parse);
    }

    #[test]
    fn unknown_session_version_falls_back_to_config_defaults() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("settings.json");
        std::fs::write(
            &path,
            r#"{"version":999,"session":{"theme_preference":"dark"}}"#,
        )
        .expect("write unsupported settings");

        let error = load_session_from_path(&path).expect_err("unsupported version should error");

        assert_eq!(error.kind(), SettingsErrorKind::UnsupportedVersion);
    }

    #[test]
    fn save_session_writes_versioned_json() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("nested").join("settings.json");
        let session = RuntimeSession::new().with_theme_preference(ThemePreference::Light);

        save_session_to_path(&path, &session).expect("settings should save");
        let loaded = load_session_from_path(&path)
            .expect("settings should load")
            .expect("settings file should exist");

        assert_eq!(loaded.theme_preference(), Some(ThemePreference::Light));
    }
}

#[cfg(feature = "file-picker")]
use std::path::PathBuf;

#[cfg(feature = "file-picker")]
use iced::Task;
#[cfg(feature = "file-picker")]
use rfd::FileDialog;

/// Filter definition shared by [`pick_file`], [`pick_files`], and
/// [`save_file`].
///
/// Available only when the `file-picker` feature is enabled. Constructing
/// `FileFilter` when the feature is disabled fails to compile, eliminating
/// the "build a params struct that goes nowhere" footgun.
///
/// Note: docs.rs badge annotation via `#[doc(cfg(feature = "file-picker"))]`
/// still requires nightly (`#![feature(doc_cfg)]`); the badge is deferred
/// until the toolchain lands it on stable. Type-level gating is the v0.1
/// source of truth.
///
/// [`pick_file`]: pick_file
/// [`pick_files`]: pick_files
/// [`save_file`]: save_file
#[cfg(feature = "file-picker")]
pub struct FileFilter {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
}

/// Parameters for [`pick_file`] and [`pick_files`]. Available only when the
/// `file-picker` crate feature is enabled.
///
/// [`pick_file`]: pick_file
/// [`pick_files`]: pick_files
#[cfg(feature = "file-picker")]
pub struct PickFileParams {
    pub filters: Vec<FileFilter>,
    pub start_dir: Option<PathBuf>,
}

/// Parameters for [`save_file`]. Available only when the `file-picker` crate
/// feature is enabled.
///
/// [`save_file`]: save_file
#[cfg(feature = "file-picker")]
pub struct SaveFileParams {
    pub filters: Vec<FileFilter>,
    pub start_dir: Option<PathBuf>,
    pub default_name: Option<String>,
}

#[cfg(feature = "file-picker")]
pub fn pick_file(params: PickFileParams) -> Task<Option<PathBuf>> {
    Task::perform(
        async move {
            let mut dialog = FileDialog::new();

            for filter in &params.filters {
                dialog = dialog.add_filter(filter.name, filter.extensions);
            }

            if let Some(start_dir) = &params.start_dir {
                dialog = dialog.set_directory(start_dir);
            }

            dialog.pick_file()
        },
        |result| result,
    )
}

#[cfg(feature = "file-picker")]
pub fn pick_files(params: PickFileParams) -> Task<Option<Vec<PathBuf>>> {
    Task::perform(
        async move {
            let mut dialog = FileDialog::new();

            for filter in &params.filters {
                dialog = dialog.add_filter(filter.name, filter.extensions);
            }

            if let Some(start_dir) = &params.start_dir {
                dialog = dialog.set_directory(start_dir);
            }

            dialog.pick_files()
        },
        |result| result,
    )
}

#[cfg(feature = "file-picker")]
pub fn pick_folder(start_dir: Option<PathBuf>) -> Task<Option<PathBuf>> {
    Task::perform(
        async move {
            let mut dialog = FileDialog::new();

            if let Some(start_dir) = &start_dir {
                dialog = dialog.set_directory(start_dir);
            }

            dialog.pick_folder()
        },
        |result| result,
    )
}

#[cfg(feature = "file-picker")]
pub fn save_file(params: SaveFileParams) -> Task<Option<PathBuf>> {
    Task::perform(
        async move {
            let mut dialog = FileDialog::new();

            for filter in &params.filters {
                dialog = dialog.add_filter(filter.name, filter.extensions);
            }

            if let Some(start_dir) = &params.start_dir {
                dialog = dialog.set_directory(start_dir);
            }

            if let Some(default_name) = &params.default_name {
                dialog = dialog.set_file_name(default_name);
            }

            dialog.save_file()
        },
        |result| result,
    )
}

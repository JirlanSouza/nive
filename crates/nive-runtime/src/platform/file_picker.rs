use std::path::PathBuf;

#[cfg(feature = "file-picker")]
use iced::Task;
#[cfg(feature = "file-picker")]
use rfd::FileDialog;

pub struct FileFilter {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
}

pub struct PickFileParams {
    pub filters: Vec<FileFilter>,
    pub start_dir: Option<PathBuf>,
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

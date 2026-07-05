use nive::prelude::*;
use nive::prelude::ui::{FileFilter, PickFileParams, SaveFileParams};
use nive::widget::column;
use std::borrow::Cow;
use std::path::PathBuf;

struct FilePickerApp;

#[derive(Debug, Clone)]
enum Message {
    PickFile,
    PickFiles,
    PickFolder,
    SaveFile,
    FilePicked(Option<PathBuf>),
    FilesPicked(Option<Vec<PathBuf>>),
    FolderPicked(Option<PathBuf>),
    FileSaved(Option<PathBuf>),
}

impl Application for FilePickerApp {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-file-picker").name("File Picker")
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (Self, ())
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        match message {
            Message::PickFile => {
                let params = PickFileParams {
                    filters: vec![FileFilter {
                        name: "Text Files",
                        extensions: &["txt", "md"],
                    }],
                    start_dir: None,
                };
                return Effect::task(nive::pick_file(params).map(Message::FilePicked));
            }
            Message::PickFiles => {
                let params = PickFileParams {
                    filters: vec![FileFilter {
                        name: "Text Files",
                        extensions: &["txt", "md"],
                    }],
                    start_dir: None,
                };
                return Effect::task(nive::pick_files(params).map(Message::FilesPicked));
            }
            Message::PickFolder => {
                return Effect::task(nive::pick_folder(None).map(Message::FolderPicked));
            }
            Message::SaveFile => {
                let params = SaveFileParams {
                    filters: vec![FileFilter {
                        name: "Text Files",
                        extensions: &["txt"],
                    }],
                    start_dir: None,
                    default_name: Some("document.txt".to_string()),
                };
                return Effect::task(nive::save_file(params).map(Message::FileSaved));
            }
            Message::FilePicked(Some(path)) => {
                return Effect::toast(Toast::success(format!("Picked file: {}", path.display())));
            }
            Message::FilesPicked(Some(paths)) => {
                return Effect::toast(Toast::success(format!("Picked {} files", paths.len())));
            }
            Message::FolderPicked(Some(path)) => {
                return Effect::toast(Toast::success(format!(
                    "Picked folder: {}",
                    path.display()
                )));
            }
            Message::FileSaved(Some(path)) => {
                return Effect::toast(Toast::success(format!("Saved to: {}", path.display())));
            }
            _ => {}
        }
        Effect::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let content = column![
            text("File Picker Example").size(24),
            text("Demonstrates native file picker dialogs"),
            button("Pick File").on_press(Message::PickFile),
            button("Pick Files").on_press(Message::PickFiles),
            button("Pick Folder").on_press(Message::PickFolder),
            button("Save File").on_press(Message::SaveFile),
            text("Each action shows a toast with the result"),
        ]
        .padding(40)
        .spacing(16);

        ScreenView::new(content)
    }

    fn window_title<'a>(
        &'a self,
        _context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> impl Into<Cow<'a, str>> + 'a {
        Cow::Borrowed("File Picker")
    }
}

fn main() -> nive::Result {
    nive::run::<FilePickerApp>()
}

use iced::window;
use nive_core::{ToastPresentation, ToastTone};

use super::{Toast, ToastId, ToastItem};

impl<Message> ToastItem<Message> {
    pub fn id(&self) -> ToastId {
        self.id
    }

    pub fn request(&self) -> &Toast<Message> {
        &self.request
    }

    /// The window that originated this toast, if it was scoped to one.
    pub fn origin(&self) -> Option<window::Id> {
        self.origin
    }
}

impl<Message> ToastPresentation for ToastItem<Message> {
    type Id = ToastId;

    fn id(&self) -> ToastId {
        ToastItem::id(self)
    }

    fn title(&self) -> &str {
        self.request().title()
    }

    fn body(&self) -> Option<&str> {
        self.request().body()
    }

    fn tone(&self) -> ToastTone {
        self.request().tone()
    }
}

impl ToastId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

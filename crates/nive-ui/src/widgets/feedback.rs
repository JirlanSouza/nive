mod indicator;
mod inline_alert;
mod loading_indicator;
mod progress_bar;
mod style;

pub use inline_alert::InlineAlert;
pub use loading_indicator::LoadingIndicator;
pub use progress_bar::ProgressBar;

pub type Callout<'a, Message> = InlineAlert<'a, Message>;
pub type Spinner<'a> = LoadingIndicator<'a>;

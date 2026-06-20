mod error_details_dialog;
mod error_empty_state;
mod error_feedback;
mod error_status_line;
mod indicator;
mod inline_alert;
mod loading_indicator;
mod operation_action_group;
mod operation_status_line;
mod presentation;
mod progress_bar;
mod resource_status_line;
mod style;

pub use error_details_dialog::ErrorDetailsDialog;
pub use error_empty_state::ErrorEmptyState;
pub use error_feedback::{
    ErrorFeedback, ErrorFeedbackAction, ErrorFeedbackActionRow, ErrorFeedbackCommandRole,
};
pub use error_status_line::ErrorStatusLine;
pub use inline_alert::InlineAlert;
pub use loading_indicator::LoadingIndicator;
pub use operation_action_group::OperationActionGroup;
pub use operation_status_line::OperationStatusLine;
pub use presentation::{
    ErrorPresentation, OperationStatusPresentation, ResourceStatusPresentation,
};
pub use progress_bar::ProgressBar;
pub use resource_status_line::ResourceStatusLine;

pub type Callout<'a, Message> = InlineAlert<'a, Message>;
pub type Spinner<'a> = LoadingIndicator<'a>;

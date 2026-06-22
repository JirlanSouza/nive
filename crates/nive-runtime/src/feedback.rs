mod error;
mod toast;

pub use error::{
    ErrorCode, InvalidErrorCode, UserFacingError, UserFacingErrorKind, UserFacingResult,
};
pub use toast::{
    Toast, ToastDuration, ToastId, ToastItem, ToastMessage, ToastPosition, ToastState, ToastTone,
};

/// Deprecated alias for [`Toast`]; kept for one release cycle (v0.1) to ease
/// migration. Will be removed in v0.2.
#[deprecated(since = "0.1.0", note = "renamed to `Toast`; will be removed in 0.2")]
pub use toast::Toast as ToastRequest;

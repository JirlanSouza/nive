use std::time::Duration;

use super::*;

fn push(state: &mut ToastState, toast: Toast, now: Instant) -> ToastId {
    state.push(toast, now, None)
}

mod queue;
mod timing;

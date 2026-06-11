//! Time-driven animation primitives.
//!
//! The timing is pure data ([`Animation`] → [`AnimationFrame`]); two widgets
//! drive it against the window clock, split by what they touch:
//!
//! - [`AnimatedVisual`] re-renders its content every frame **without** changing
//!   layout — rotation, opacity, colour, the loading dots.
//! - [`AnimatedLayout`] animates the **size** of static content, reflowing its
//!   siblings (expand/collapse, reveal).
//!
//! For animating arbitrary non-size values from your `view` (colours, offsets,
//! hover), prefer the native [`iced::animation::Animation`] directly.
//!
//! [`StaggeredPulse`] is a shared curve for sequencing several items off one
//! frame.

mod layout;
mod runner;
mod stager;
mod timeline;
mod visual;

pub use layout::AnimatedLayout;
pub use stager::StaggeredPulse;
pub use timeline::{Animation, AnimationFrame, Easing};
pub use visual::AnimatedVisual;

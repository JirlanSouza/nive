mod geometry;
#[allow(dead_code)]
pub(crate) mod scroll;

pub(crate) use geometry::{resolve_geometry, GeometryInput, SAFE_VIEWPORT_MARGIN};
pub use geometry::{PopoverCollision, PopoverPlacement, PopoverWidth};

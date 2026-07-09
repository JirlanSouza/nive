//! Shared desktop interaction foundation for reusable widgets.
//!
//! `nive_ui::interaction` is the widget-agnostic layer for pointer gestures,
//! logical selection, activation and rename triggers, context requests,
//! transfer payloads, and app-internal drag-and-drop. Collection widgets such
//! as Tree, List, and Table consume this module for shared concepts and keep
//! only geometry-specific target types in their own modules.
//!
//! The foundation owns interaction semantics and normalized intent shapes.
//! Applications own domain mutation. For example, a successful move drop emits
//! a [`DropCommit`], but the app decides whether and when to remove source
//! data, create target data, persist changes, or ignore the request.
//!
//! Intent enums are marked `#[non_exhaustive]` when future variants should be
//! non-breaking. Apps should include a wildcard arm when matching them. Public
//! structs that apps naturally construct expose fields or constructors for
//! normal application and test usage.
//!
//! # Component Library To Canvas
//!
//! ```
//! use nive_ui::interaction::{
//!     Drag, DropContext, DropDecision, DropCommit, TransferData, TransferOperation,
//!     TransferOperations,
//! };
//!
//! #[derive(Clone, Debug, PartialEq)]
//! enum AppPayload {
//!     Component(&'static str),
//! }
//!
//! #[derive(Clone, Debug, PartialEq)]
//! enum AppOrigin {
//!     Library,
//! }
//!
//! #[derive(Clone, Debug, PartialEq)]
//! enum CanvasTarget {
//!     Point(iced::Point),
//! }
//!
//! let drag = Drag {
//!     payload: TransferData::local(AppPayload::Component("button")),
//!     origin: AppOrigin::Library,
//!     operations: TransferOperations::COPY,
//!     preferred: TransferOperation::Copy,
//! };
//!
//! let context = DropContext {
//!     payload: &drag.payload,
//!     origin: &drag.origin,
//!     operations: drag.operations,
//!     preferred: drag.preferred,
//!     requested: None,
//!     position: iced::Point::new(16.0, 24.0),
//!     modifiers: iced::keyboard::Modifiers::NONE,
//! };
//!
//! let decision = DropDecision::accept(CanvasTarget::Point(context.position), TransferOperation::Copy);
//! let DropDecision::Accept { target, operation } = decision else {
//!     unreachable!("canvas accepts component payloads");
//! };
//! let position = context.position;
//!
//! let commit = DropCommit {
//!     payload: drag.payload,
//!     origin: drag.origin,
//!     target,
//!     operation,
//!     position,
//! };
//!
//! assert_eq!(commit.operation, TransferOperation::Copy);
//! ```
//!
//! # Collection Reorder
//!
//! ```
//! use nive_ui::interaction::{
//!     CollectionTransferPayload, Drag, DropCommit, TransferData, TransferOperation,
//!     TransferOperations,
//! };
//!
//! #[derive(Clone, Debug, PartialEq)]
//! enum ListTarget {
//!     Before(u32),
//! }
//!
//! let payload = CollectionTransferPayload::flat([2, 3]);
//! let drag = Drag {
//!     payload: TransferData::local(payload),
//!     origin: "left-list",
//!     operations: TransferOperations::MOVE,
//!     preferred: TransferOperation::Move,
//! };
//!
//! let commit = DropCommit {
//!     payload: drag.payload,
//!     origin: drag.origin,
//!     target: ListTarget::Before(4),
//!     operation: TransferOperation::Move,
//!     position: iced::Point::ORIGIN,
//! };
//!
//! if commit.operation == TransferOperation::Move {
//!     // The app mutates its model after receiving the commit.
//! }
//! ```

pub mod context;
pub mod dnd;
pub mod keyboard;
pub mod orientation;
pub mod pointer;
pub mod selection;
pub mod transfer;

pub use context::{
    ContextInvocation, ContextPosition, ContextRequest, ContextSelectionBehavior, ContextTarget,
};
pub use dnd::{linear_insertion, Drag, DropCommit, DropContext, DropDecision, LinearInsertion};
pub use keyboard::{ActivationBehavior, ActivationTrigger, RenameBehavior, StepAdjustment};
pub use orientation::Orientation;
pub use pointer::{PointerButton, PointerGesture, PointerGestureKind, PointerGestureState};
pub use selection::{ClickModifiers, Selection, SelectionMode, SelectionSnapshot};
pub use transfer::{
    CollectionTransferPayload, Transfer, TransferData, TransferOperation, TransferOperations,
};

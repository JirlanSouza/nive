use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio_util::sync::CancellationToken;

static NEXT_SCOPE_ID: AtomicU64 = AtomicU64::new(1);

fn next_scope_id() -> NonZeroU64 {
    let value = NEXT_SCOPE_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            (current != 0).then_some(current.checked_add(1).unwrap_or(0))
        })
        .unwrap_or_else(|_| panic!("Nive task scope identity space exhausted"));
    NonZeroU64::new(value).expect("scope identity is never zero")
}

struct ScopeRegistration {
    key: NonZeroU64,
    parent: Option<NonZeroU64>,
    label: Cow<'static, str>,
    cancellation: CancellationToken,
}

/// Opaque capability identifying a structured task lifetime.
#[derive(Clone)]
pub struct ScopeId(Arc<ScopeRegistration>);

impl ScopeId {
    pub(crate) fn root(label: impl Into<Cow<'static, str>>) -> TaskScope {
        TaskScope {
            id: Self(Arc::new(ScopeRegistration {
                key: next_scope_id(),
                parent: None,
                label: label.into(),
                cancellation: CancellationToken::new(),
            })),
        }
    }

    /// Creates an RAII-owned child lifetime.
    pub fn child(&self, label: impl Into<Cow<'static, str>>) -> TaskScope {
        TaskScope {
            id: Self(Arc::new(ScopeRegistration {
                key: next_scope_id(),
                parent: Some(self.0.key),
                label: label.into(),
                cancellation: self.0.cancellation.child_token(),
            })),
        }
    }

    pub(crate) fn token(&self) -> CancellationToken {
        self.0.cancellation.clone()
    }

    pub(crate) fn key(&self) -> NonZeroU64 {
        self.0.key
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.0.cancellation.is_cancelled()
    }
}

impl fmt::Debug for ScopeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScopeId")
            .field("label", &self.0.label)
            .field("has_parent", &self.0.parent.is_some())
            .field("closed", &self.is_closed())
            .finish_non_exhaustive()
    }
}

impl PartialEq for ScopeId {
    fn eq(&self, other: &Self) -> bool {
        self.0.key == other.0.key
    }
}

impl Eq for ScopeId {}

impl Hash for ScopeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.key.hash(state);
    }
}

/// RAII owner of a child task scope. Dropping it cancels all descendants.
#[must_use = "keep the task scope alive for as long as its owner"]
pub struct TaskScope {
    id: ScopeId,
}

impl TaskScope {
    /// Returns a cloneable capability for admitting requests into this scope.
    pub fn id(&self) -> ScopeId {
        self.id.clone()
    }

    /// Creates an RAII-owned child lifetime.
    pub fn child(&self, label: impl Into<Cow<'static, str>>) -> Self {
        self.id.child(label)
    }

    /// Closes this lifetime and all of its descendants.
    pub fn close(self) {}

    pub(crate) fn cancel(&self) {
        self.id.0.cancellation.cancel();
    }
}

impl fmt::Debug for TaskScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("TaskScope").field(&self.id).finish()
    }
}

impl Drop for TaskScope {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_child_cancels_descendants_but_not_parent_or_sibling() {
        let root = ScopeId::root("app");
        let child = root.child("screen");
        let grandchild = child.child("panel");
        let sibling = root.child("other");

        child.close();

        assert!(!root.id().is_closed());
        assert!(grandchild.id().is_closed());
        assert!(!sibling.id().is_closed());
    }

    #[test]
    fn closing_parent_cancels_every_descendant() {
        let root = ScopeId::root("app");
        let child = root.child("window");
        let grandchild = child.child("screen");

        root.cancel();

        assert!(child.id().is_closed());
        assert!(grandchild.id().is_closed());
    }
}

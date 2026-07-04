use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};

/// Transfer operation requested by clipboard or drag/drop interactions.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum TransferOperation {
    /// Copy the payload.
    Copy,
    /// Move the payload.
    #[default]
    Move,
    /// Link the payload.
    Link,
}

/// Set of operations a transfer source allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TransferOperations(u8);

impl TransferOperations {
    /// No operations are allowed.
    pub const NONE: Self = Self(0);
    /// Copy is allowed.
    pub const COPY: Self = Self(0b001);
    /// Move is allowed.
    pub const MOVE: Self = Self(0b010);
    /// Link is allowed.
    pub const LINK: Self = Self(0b100);
    /// Copy, move, and link are allowed.
    pub const ALL: Self = Self(Self::COPY.0 | Self::MOVE.0 | Self::LINK.0);

    /// Builds a set containing one operation.
    pub const fn from_operation(operation: TransferOperation) -> Self {
        match operation {
            TransferOperation::Copy => Self::COPY,
            TransferOperation::Move => Self::MOVE,
            TransferOperation::Link => Self::LINK,
        }
    }

    /// Returns whether the operation is allowed.
    pub const fn contains(self, operation: TransferOperation) -> bool {
        self.0 & Self::from_operation(operation).0 != 0
    }

    /// Returns a set with the operation added.
    pub const fn with(self, operation: TransferOperation) -> Self {
        Self(self.0 | Self::from_operation(operation).0)
    }

    /// Returns whether no operations are allowed.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the first allowed operation in stable preference order.
    pub const fn first(self) -> Option<TransferOperation> {
        if self.contains(TransferOperation::Copy) {
            Some(TransferOperation::Copy)
        } else if self.contains(TransferOperation::Move) {
            Some(TransferOperation::Move)
        } else if self.contains(TransferOperation::Link) {
            Some(TransferOperation::Link)
        } else {
            None
        }
    }

    /// Chooses `requested`, then `preferred`, then the first allowed operation.
    pub const fn preferred(
        self,
        requested: Option<TransferOperation>,
        preferred: TransferOperation,
    ) -> Option<TransferOperation> {
        if let Some(requested) = requested {
            if self.contains(requested) {
                return Some(requested);
            }
        }

        if self.contains(preferred) {
            Some(preferred)
        } else {
            self.first()
        }
    }
}

impl From<TransferOperation> for TransferOperations {
    fn from(operation: TransferOperation) -> Self {
        Self::from_operation(operation)
    }
}

impl std::ops::BitOr for TransferOperations {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for TransferOperations {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Transfer data shared by clipboard and drag/drop flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferData<T> {
    /// App-local typed payload.
    Local(T),
    /// File paths reserved for file transfer adapters.
    Files(Vec<PathBuf>),
    /// Text payload.
    Text(String),
    /// MIME-tagged bytes.
    Bytes {
        /// MIME type for the byte payload.
        mime: Cow<'static, str>,
        /// Shared immutable bytes.
        bytes: Arc<[u8]>,
    },
}

impl<T> TransferData<T> {
    /// Builds app-local transfer data.
    pub fn local(payload: T) -> Self {
        Self::Local(payload)
    }

    /// Builds file transfer data.
    pub fn files(paths: impl IntoIterator<Item = PathBuf>) -> Self {
        Self::Files(paths.into_iter().collect())
    }

    /// Builds text transfer data.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// Builds MIME byte transfer data.
    pub fn bytes(mime: impl Into<Cow<'static, str>>, bytes: impl Into<Arc<[u8]>>) -> Self {
        Self::Bytes {
            mime: mime.into(),
            bytes: bytes.into(),
        }
    }
}

/// Deterministic selected-item payload for collection transfer intents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionTransferPayload<Id> {
    /// Payload IDs in widget-visible order.
    pub ids: Vec<Id>,
    /// Hierarchy-normalized root IDs. Flat collections use the same values as
    /// [`ids`](Self::ids).
    pub root_ids: Vec<Id>,
}

impl<Id> CollectionTransferPayload<Id> {
    /// Builds a collection transfer payload from selected and root IDs.
    pub fn new(ids: Vec<Id>, root_ids: Vec<Id>) -> Self {
        Self { ids, root_ids }
    }

    /// Builds a flat collection payload where all selected IDs are roots.
    pub fn flat(ids: impl IntoIterator<Item = Id>) -> Self
    where
        Id: Clone,
    {
        let ids: Vec<Id> = ids.into_iter().collect();

        Self {
            root_ids: ids.clone(),
            ids,
        }
    }
}

impl<Id> CollectionTransferPayload<Id>
where
    Id: Clone + Ord,
{
    /// Builds a flat collection payload from visible IDs and selected IDs.
    pub fn from_visible_order(
        visible_ids: impl IntoIterator<Item = Id>,
        selected_ids: &BTreeSet<Id>,
    ) -> Self {
        let ids: Vec<Id> = visible_ids
            .into_iter()
            .filter(|id| selected_ids.contains(id))
            .collect();

        Self::flat(ids)
    }

    /// Builds a hierarchical payload and removes selected descendants from
    /// `root_ids`.
    pub fn from_visible_order_with_parents(
        visible_ids: impl IntoIterator<Item = Id>,
        selected_ids: &BTreeSet<Id>,
        parent_by_id: impl IntoIterator<Item = (Id, Option<Id>)>,
    ) -> Self {
        let ids: Vec<Id> = visible_ids
            .into_iter()
            .filter(|id| selected_ids.contains(id))
            .collect();
        let parent_by_id: BTreeMap<Id, Option<Id>> = parent_by_id.into_iter().collect();
        let selected: BTreeSet<Id> = ids.iter().cloned().collect();
        let root_ids = ids
            .iter()
            .filter(|id| !has_selected_ancestor(*id, &parent_by_id, &selected))
            .cloned()
            .collect();

        Self { ids, root_ids }
    }
}

/// Visual transfer state shared by collection widgets.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Transfer<Id, Target> {
    /// No transfer feedback.
    #[default]
    None,
    /// Clipboard feedback such as cut state.
    Clipboard {
        /// Clipboard payload.
        payload: CollectionTransferPayload<Id>,
        /// Clipboard operation.
        operation: TransferOperation,
    },
    /// Active drag feedback.
    Dragging {
        /// Drag payload.
        payload: CollectionTransferPayload<Id>,
        /// Drag operation.
        operation: TransferOperation,
        /// Current drop target, if any.
        target: Option<Target>,
    },
}

fn has_selected_ancestor<Id>(
    id: &Id,
    parent_by_id: &BTreeMap<Id, Option<Id>>,
    selected: &BTreeSet<Id>,
) -> bool
where
    Id: Clone + Ord,
{
    let mut parent = parent_by_id.get(id).and_then(Clone::clone);
    while let Some(id) = parent {
        if selected.contains(&id) {
            return true;
        }
        parent = parent_by_id.get(&id).and_then(Clone::clone);
    }
    false
}

#[cfg(test)]
mod transfer_tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn operation_sets_contain_and_choose_operations() {
        let operations = TransferOperations::COPY | TransferOperations::MOVE;

        assert!(operations.contains(TransferOperation::Copy));
        assert!(operations.contains(TransferOperation::Move));
        assert!(!operations.contains(TransferOperation::Link));
        assert_eq!(
            operations.preferred(Some(TransferOperation::Copy), TransferOperation::Move),
            Some(TransferOperation::Copy)
        );
        assert_eq!(
            operations.preferred(Some(TransferOperation::Link), TransferOperation::Move),
            Some(TransferOperation::Move)
        );
        assert!(TransferOperations::NONE.is_empty());
    }

    #[test]
    fn transfer_data_constructors_build_expected_variants() {
        assert_eq!(TransferData::local(12), TransferData::Local(12));
        assert_eq!(
            TransferData::<()>::text("hello"),
            TransferData::Text(String::from("hello"))
        );
        assert_eq!(
            TransferData::<()>::files([PathBuf::from("/tmp/a")]),
            TransferData::Files(vec![PathBuf::from("/tmp/a")])
        );
        assert_eq!(
            TransferData::<()>::bytes("text/plain", Arc::<[u8]>::from([1, 2])),
            TransferData::Bytes {
                mime: Cow::Borrowed("text/plain"),
                bytes: Arc::<[u8]>::from([1, 2])
            }
        );
    }

    #[test]
    fn collection_payload_preserves_visible_order() {
        let selected = [3, 1].into_iter().collect();
        let payload = CollectionTransferPayload::from_visible_order([1, 2, 3], &selected);

        assert_eq!(payload.ids, vec![1, 3]);
        assert_eq!(payload.root_ids, vec![1, 3]);
    }

    #[test]
    fn hierarchical_collection_payload_normalizes_root_ids() {
        let selected = ["root", "a", "c"].into_iter().collect();
        let parents = [
            ("root", None),
            ("a", Some("root")),
            ("b", Some("root")),
            ("c", Some("b")),
        ];

        let payload = CollectionTransferPayload::from_visible_order_with_parents(
            ["root", "a", "b", "c"],
            &selected,
            parents,
        );

        assert_eq!(payload.ids, vec!["root", "a", "c"]);
        assert_eq!(payload.root_ids, vec!["root"]);
    }

    #[test]
    fn flat_collection_payload_degenerates_to_selected_ids() {
        let payload = CollectionTransferPayload::flat(["a", "b"]);

        assert_eq!(payload.ids, vec!["a", "b"]);
        assert_eq!(payload.root_ids, vec!["a", "b"]);
    }
}

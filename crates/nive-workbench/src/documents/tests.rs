use nive_ui::IconRole;

use super::*;

#[test]
fn maps_document_metadata_to_tab_item() {
    let item = WorkbenchDocument::new("main", "main.rs")
        .icon(IconRole::Folder)
        .dirty(true)
        .pinned(true)
        .closable(false)
        .disabled(true)
        .tooltip("generated")
        .into_tab_item();

    assert_eq!(item.id(), &"main");
    assert_eq!(item.label(), "main.rs");
}

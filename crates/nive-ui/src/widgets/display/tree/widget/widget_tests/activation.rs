use super::support::*;

#[test]
fn tree_should_activate_delegates_to_behavior() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .activation_behavior(ActivationBehavior::EnterAndDoubleClick);

    assert!(tree.should_activate(ActivationTrigger::Enter));
    assert!(tree.should_activate(ActivationTrigger::DoubleClick));
    assert!(!tree.should_activate(ActivationTrigger::Space));
}

#[test]
fn tree_should_rename_delegates_to_behavior() {
    use iced::keyboard::key::Named;

    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .rename_behavior(RenameBehavior::F2);

    assert!(tree.should_rename(Named::F2));
    assert!(!tree.should_rename(Named::Enter));
}

#[test]
fn tree_should_rename_with_return_behavior() {
    use iced::keyboard::key::Named;

    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .rename_behavior(RenameBehavior::Return);

    assert!(tree.should_rename(Named::Enter));
    assert!(!tree.should_rename(Named::F2));
}

#[test]
fn tree_should_rename_with_platform_behavior() {
    use iced::keyboard::key::Named;

    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new());

    if cfg!(target_os = "macos") {
        assert!(tree.should_rename(Named::Enter));
        assert!(!tree.should_rename(Named::F2));
    } else {
        assert!(tree.should_rename(Named::F2));
        assert!(!tree.should_rename(Named::Enter));
    }
}

#[test]
fn tree_should_activate_with_platform_behavior() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new());

    assert!(tree.should_activate(ActivationTrigger::DoubleClick));

    if cfg!(target_os = "macos") {
        assert!(tree.should_activate(ActivationTrigger::Space));
        assert!(tree.should_activate(ActivationTrigger::CommandO));
        assert!(tree.should_activate(ActivationTrigger::CommandDown));
        assert!(!tree.should_activate(ActivationTrigger::Enter));
    } else {
        assert!(tree.should_activate(ActivationTrigger::Enter));
        assert!(!tree.should_activate(ActivationTrigger::Space));
        assert!(!tree.should_activate(ActivationTrigger::CommandO));
    }
}

#[test]
fn tree_should_activate_with_enter_behavior() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .activation_behavior(ActivationBehavior::Enter);

    assert!(tree.should_activate(ActivationTrigger::Enter));
    assert!(!tree.should_activate(ActivationTrigger::Space));
    assert!(!tree.should_activate(ActivationTrigger::DoubleClick));
    assert!(!tree.should_activate(ActivationTrigger::CommandO));
    assert!(!tree.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn tree_should_activate_with_space_behavior() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .activation_behavior(ActivationBehavior::Space);

    assert!(tree.should_activate(ActivationTrigger::Space));
    assert!(!tree.should_activate(ActivationTrigger::Enter));
    assert!(!tree.should_activate(ActivationTrigger::DoubleClick));
    assert!(!tree.should_activate(ActivationTrigger::CommandO));
    assert!(!tree.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn tree_should_activate_with_double_click_only_behavior() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .activation_behavior(ActivationBehavior::DoubleClick);

    assert!(tree.should_activate(ActivationTrigger::DoubleClick));
    assert!(!tree.should_activate(ActivationTrigger::Enter));
    assert!(!tree.should_activate(ActivationTrigger::Space));
    assert!(!tree.should_activate(ActivationTrigger::CommandO));
    assert!(!tree.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn tree_should_activate_with_enter_and_double_click_behavior() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .activation_behavior(ActivationBehavior::EnterAndDoubleClick);

    assert!(tree.should_activate(ActivationTrigger::Enter));
    assert!(tree.should_activate(ActivationTrigger::DoubleClick));
    assert!(!tree.should_activate(ActivationTrigger::Space));
    assert!(!tree.should_activate(ActivationTrigger::CommandO));
    assert!(!tree.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn tree_should_activate_with_space_and_double_click_behavior() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .activation_behavior(ActivationBehavior::SpaceAndDoubleClick);

    assert!(tree.should_activate(ActivationTrigger::Space));
    assert!(tree.should_activate(ActivationTrigger::DoubleClick));
    assert!(!tree.should_activate(ActivationTrigger::Enter));
    assert!(!tree.should_activate(ActivationTrigger::CommandO));
    assert!(!tree.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn tree_should_activate_with_enter_space_and_double_click_behavior() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .activation_behavior(ActivationBehavior::EnterSpaceAndDoubleClick);

    assert!(tree.should_activate(ActivationTrigger::Enter));
    assert!(tree.should_activate(ActivationTrigger::Space));
    assert!(tree.should_activate(ActivationTrigger::DoubleClick));
    assert!(!tree.should_activate(ActivationTrigger::CommandO));
    assert!(!tree.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn tree_should_activate_with_command_open_and_double_click_behavior() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .activation_behavior(ActivationBehavior::CommandOpenAndDoubleClick);

    assert!(tree.should_activate(ActivationTrigger::CommandO));
    assert!(tree.should_activate(ActivationTrigger::CommandDown));
    assert!(tree.should_activate(ActivationTrigger::DoubleClick));
    assert!(!tree.should_activate(ActivationTrigger::Enter));
    assert!(!tree.should_activate(ActivationTrigger::Space));
}

#[test]
fn tree_should_activate_with_space_command_open_and_double_click_behavior() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .activation_behavior(ActivationBehavior::SpaceCommandOpenAndDoubleClick);

    assert!(tree.should_activate(ActivationTrigger::Space));
    assert!(tree.should_activate(ActivationTrigger::CommandO));
    assert!(tree.should_activate(ActivationTrigger::CommandDown));
    assert!(tree.should_activate(ActivationTrigger::DoubleClick));
    assert!(!tree.should_activate(ActivationTrigger::Enter));
}

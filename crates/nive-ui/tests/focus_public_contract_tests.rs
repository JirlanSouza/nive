#[test]
fn managed_focus_paths_are_explicit_and_support_downstream_widgets() {
    use iced::advanced::{
        layout::{Limits, Node},
        widget::{operation, tree, Tree},
        Layout, Widget,
    };
    use nive_ui::{
        accessibility::FocusRoot,
        advanced::focus::{FocusState, FocusVisibility},
    };

    struct CustomFocusTarget;

    impl Widget<(), nive_ui::Theme, nive_ui::Renderer> for CustomFocusTarget {
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<FocusState>()
        }

        fn state(&self) -> tree::State {
            tree::State::new(FocusState::new(FocusVisibility::Auto))
        }

        fn size(&self) -> iced::Size<iced::Length> {
            iced::Size::new(iced::Length::Fixed(80.0), iced::Length::Fixed(28.0))
        }

        fn layout(
            &mut self,
            _tree: &mut Tree,
            _renderer: &nive_ui::Renderer,
            limits: &Limits,
        ) -> Node {
            Node::new(limits.resolve(
                iced::Length::Fixed(80.0),
                iced::Length::Fixed(28.0),
                iced::Size::new(80.0, 28.0),
            ))
        }

        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            _renderer: &nive_ui::Renderer,
            operation: &mut dyn operation::Operation,
        ) {
            tree.state
                .downcast_mut::<FocusState>()
                .register(operation, None, layout.bounds());
        }

        fn draw(
            &self,
            _tree: &Tree,
            _renderer: &mut nive_ui::Renderer,
            _theme: &nive_ui::Theme,
            _style: &iced::advanced::renderer::Style,
            _layout: Layout<'_>,
            _cursor: iced::advanced::mouse::Cursor,
            _viewport: &iced::Rectangle,
        ) {
        }
    }

    let target: nive_ui::Element<'_, ()> = nive_ui::Element::new(CustomFocusTarget);
    let _: nive_ui::Element<'_, ()> = FocusRoot::new(target).into();
    let state = FocusState::new(FocusVisibility::AlwaysWhileActive);
    assert!(!state.is_active());
    assert!(!state.is_focus_visible());

    let prelude_source = include_str!("../src/prelude.rs");
    assert!(!prelude_source.contains("FocusRoot"));
    assert!(!prelude_source.contains("FocusState"));
    assert!(!prelude_source.contains("FocusVisibility"));
}

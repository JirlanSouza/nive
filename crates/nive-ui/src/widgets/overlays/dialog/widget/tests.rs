use super::seam::SEAM_VISIBILITY_EPSILON;
use super::*;

fn layout_dialog<Message: Clone + 'static>(
    dialog: Dialog<'static, Message>,
    max: Size,
) -> layout::Node {
    crate::test_support::layout(dialog.into(), max)
}

/// `layout` measures the footer against a throwaway `Tree::new(&*footer)` to
/// reserve its height. That fresh tree is built from the footer's *current*
/// elements, which a previous narrow pass may have left truncated — so a
/// child tree can arrive describing a `Tooltip` while its fresh state says
/// nothing is truncated, and the unbounded measuring pass then lays a plain
/// `Text` against it.
///
/// Reaching it needs the real resize path: `relayout` re-runs `layout` on
/// the retained tree with no `diff` in between.
#[test]
fn remeasuring_a_footer_after_a_narrow_pass_does_not_mismatch_its_child_trees() {
    use crate::test_support::WidgetHarness;
    use crate::widgets::overlays::dialog::footer::{
        DialogAction, DialogActionFooter, DialogTerminalAction,
    };

    let dialog = || -> Element<'static, ()> {
        Dialog::new(iced::widget::text("Body"))
            .footer(DialogActionFooter::with_one(
                DialogAction::cancel("Discard every unsaved change", ()),
                DialogTerminalAction::primary("Save and close this document now", ()),
            ))
            .into()
    };

    let mut harness = WidgetHarness::new(dialog(), Size::new(900.0, 900.0));
    harness.draw();

    // Narrow first, so the action labels truncate and the footer's elements
    // are left holding tooltip-wrapped content.
    for width in [200.0, 900.0, 160.0, 900.0] {
        harness.relayout(Size::new(width, 900.0));
        harness.draw();
    }
}

#[test]
fn header_seam_is_hidden_at_the_top_and_visible_once_scrolled() {
    assert!(!header_seam_visible(0.0));
    assert!(!header_seam_visible(SEAM_VISIBILITY_EPSILON));
    assert!(header_seam_visible(12.0));
}

#[test]
fn footer_seam_is_visible_only_while_content_remains_below() {
    // No overflow at all: never visible, regardless of offset.
    assert!(!footer_seam_visible(0.0, 200.0, 200.0));

    // Overflowing content, unscrolled: visible.
    assert!(footer_seam_visible(0.0, 600.0, 200.0));

    // Scrolled to the exact bottom: hidden again.
    assert!(!footer_seam_visible(400.0, 600.0, 200.0));

    // Scrolled partway through overflowing content: still visible.
    assert!(footer_seam_visible(150.0, 600.0, 200.0));
}

#[test]
fn resolves_semantic_widths_in_an_unconstrained_viewport() {
    for (size, expected) in [
        (DialogSize::Sm, 420.0),
        (DialogSize::Md, 560.0),
        (DialogSize::Lg, 720.0),
    ] {
        let dialog = Dialog::<()>::new(iced::widget::text("Body")).size(size);
        let node = layout_dialog(dialog, Size::new(2000.0, 2000.0));

        assert_eq!(node.size().width, expected);
    }
}

#[test]
fn default_size_is_sm() {
    let dialog = Dialog::<()>::new(iced::widget::text("Body"));
    let node = layout_dialog(dialog, Size::new(2000.0, 2000.0));

    assert_eq!(node.size().width, 420.0);
}

#[test]
fn width_clamps_to_the_limits_max_width() {
    let dialog = Dialog::<()>::new(iced::widget::text("Body")).size(DialogSize::Lg);
    let node = layout_dialog(dialog, Size::new(300.0, 2000.0));

    assert_eq!(node.size().width, 300.0);
}

#[test]
fn body_and_slot_insets_grow_with_density() {
    use crate::theme::{testing::ThemeTestGuard, ThemeBuilder, ThemeMode};

    fn dialog_height(density: crate::theme::ThemeDensity) -> f32 {
        let theme = ThemeBuilder::new("Density test", ThemeMode::Dark)
            .density(density)
            .build();
        let _guard = ThemeTestGuard::activate(theme);

        let dialog = Dialog::<()>::new(iced::widget::text("Body"))
            .header(iced::widget::text("Title"))
            .footer(iced::widget::text("Footer"));
        layout_dialog(dialog, Size::new(420.0, 2000.0))
            .size()
            .height
    }

    let compact = dialog_height(crate::theme::ThemeDensity::Compact);
    let comfortable = dialog_height(crate::theme::ThemeDensity::Comfortable);

    assert!(
        compact < comfortable,
        "compact height {compact} should be shorter than comfortable height {comfortable}"
    );
}

#[test]
fn total_height_never_exceeds_the_limits_max_height() {
    let long_body =
        iced::widget::column((0..200).map(|i| iced::widget::text(format!("Line {i}")).into()));
    let dialog = Dialog::<()>::new(long_body)
        .header(iced::widget::text("Title"))
        .footer(iced::widget::text("Footer"));
    let node = layout_dialog(dialog, Size::new(420.0, 400.0));

    assert!(node.size().height <= 400.0 + f32::EPSILON);
}

#[test]
fn short_content_does_not_stretch_to_the_available_cap() {
    let dialog =
        Dialog::<()>::new(iced::widget::text("One line")).header(iced::widget::text("Title"));
    let node = layout_dialog(dialog, Size::new(420.0, 4000.0));

    assert!(node.size().height < 400.0);
}

#[test]
fn header_and_footer_are_positioned_before_and_after_the_body() {
    let dialog = Dialog::<()>::new(iced::widget::text("Body"))
        .header(iced::widget::text("Title"))
        .footer(iced::widget::text("Footer"));
    let node = layout_dialog(dialog, Size::new(420.0, 2000.0));
    let children: Vec<_> = node.children().to_vec();

    assert_eq!(children.len(), 3);
    assert_eq!(children[0].bounds().y, 0.0);
    assert!(children[1].bounds().y >= children[0].bounds().y + children[0].bounds().height);
    assert!(children[2].bounds().y >= children[1].bounds().y + children[1].bounds().height);
}

#[test]
fn body_consumed_enter_prevents_the_footer_default_from_also_firing() {
    use super::super::{DialogActionFooter, DialogTerminalAction};
    use iced::keyboard;

    let key = keyboard::Key::Named(keyboard::key::Named::Enter);
    let event = Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Enter),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    });

    let dialog = Dialog::new(AlwaysCapture).footer(DialogActionFooter::new(
        DialogTerminalAction::primary("Save", "save"),
    ));

    let messages: Vec<&'static str> =
        crate::test_support::event_messages(dialog.into(), Size::new(420.0, 400.0), event);

    assert!(messages.is_empty());
}

/// Minimal test-only widget that captures every event, standing in for a
/// text editor or nested overlay that consumes Enter itself.
struct AlwaysCapture;

impl<Message> Widget<Message, Theme, Renderer> for AlwaysCapture {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(10.0), Length::Fixed(10.0))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.resolve(
            Length::Fixed(10.0),
            Length::Fixed(10.0),
            Size::new(10.0, 10.0),
        ))
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        shell.capture_event();
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _inherited_style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }
}

impl<'a, Message: 'a> From<AlwaysCapture> for Element<'a, Message> {
    fn from(widget: AlwaysCapture) -> Self {
        Element::new(widget)
    }
}

use iced::{widget::text, Color};

use crate::theme::{text as theme_text, TextRole, ToneRole, TypographyRole};

pub fn heading<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::Heading, TextRole::Primary)
}

pub fn title<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::Title, TextRole::Primary)
}

pub fn section_label<'a>(
    content: impl text::IntoFragment<'a>,
) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::SectionLabel, TextRole::Primary)
}

pub fn body<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::Body, TextRole::Primary)
}

/// Emphasized 14 px ordinary content, suitable for card titles.
pub fn body_strong<'a>(
    content: impl text::IntoFragment<'a>,
) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::BodyStrong, TextRole::Primary)
}

pub fn body_small<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::BodySmall, TextRole::Secondary)
}

pub fn label<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::Label, TextRole::Secondary)
}

pub fn label_strong<'a>(
    content: impl text::IntoFragment<'a>,
) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::LabelStrong, TextRole::Primary)
}

pub fn badge_label<'a>(
    content: impl text::IntoFragment<'a>,
) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::BadgeLabel, TextRole::Primary)
}

pub fn caption<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::Caption, TextRole::Muted)
}

pub fn code<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::Code, TextRole::Secondary)
}

pub fn code_small<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::CodeSmall, TextRole::Secondary)
}

pub fn metadata_tag<'a>(
    content: impl text::IntoFragment<'a>,
) -> text::Text<'a, crate::theme::Theme> {
    with_role(content, TypographyRole::MetadataTag, TextRole::Secondary)
}

pub fn with_role<'a>(
    content: impl text::IntoFragment<'a>,
    style_role: TypographyRole,
    color_role: TextRole,
) -> text::Text<'a, crate::theme::Theme> {
    with_typography(content, style_role).style(theme_text::style(color_role))
}

pub(crate) fn with_typography<'a, Renderer>(
    content: impl text::IntoFragment<'a>,
    style_role: TypographyRole,
) -> text::Text<'a, crate::theme::Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    text(content)
        .font(theme_text::font_for_role(style_role))
        .size(theme_text::size_for_role(style_role))
        .line_height(theme_text::line_height_for_role(style_role))
        .shaping(text::Shaping::Auto)
}

#[cfg(test)]
fn styled_text<'a, Renderer>(
    content: impl text::IntoFragment<'a>,
    style_role: TypographyRole,
    color_role: TextRole,
) -> text::Text<'a, crate::theme::Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    with_typography(content, style_role).style(theme_text::style(color_role))
}

pub fn text_color(color: Color) -> impl Fn(&crate::theme::Theme) -> iced::widget::text::Style {
    theme_text::color(color)
}

pub fn text_primary() -> impl Fn(&crate::theme::Theme) -> iced::widget::text::Style {
    theme_text::style(TextRole::Primary)
}

pub fn text_secondary() -> impl Fn(&crate::theme::Theme) -> iced::widget::text::Style {
    theme_text::style(TextRole::Secondary)
}

pub fn text_muted() -> impl Fn(&crate::theme::Theme) -> iced::widget::text::Style {
    theme_text::style(TextRole::Muted)
}

pub fn text_accent() -> impl Fn(&crate::theme::Theme) -> iced::widget::text::Style {
    theme_text::tone(ToneRole::Accent)
}

#[cfg(test)]
mod text_wrapper_tests {
    use super::*;
    use iced::advanced::{layout, renderer, text as advanced_text, widget, Widget};
    use iced::{
        alignment, Background, Color, Font, Pixels, Point, Rectangle, Size, Transformation,
    };

    #[test]
    fn styled_text_applies_typography_and_auto_shaping_to_app_text() {
        let mut widget =
            styled_text::<RecordingRenderer>("Body", TypographyRole::Body, TextRole::Primary);
        let mut tree =
            widget::Tree::new(&widget as &dyn Widget<(), crate::theme::Theme, RecordingRenderer>);
        let renderer = RecordingRenderer;

        let _node = <text::Text<'_, crate::theme::Theme, RecordingRenderer> as Widget<
            (),
            crate::theme::Theme,
            RecordingRenderer,
        >>::layout(&mut widget, &mut tree, &renderer, &layout::Limits::NONE);

        let state = tree.state.downcast_ref::<text::State<RecordingParagraph>>();

        assert_eq!(
            state.raw().line_height,
            theme_text::line_height_for_role(TypographyRole::Body)
        );
        assert_eq!(state.raw().shaping, advanced_text::Shaping::Auto);
    }

    #[test]
    fn body_strong_applies_the_complete_semantic_text_style() {
        let mut widget = styled_text::<RecordingRenderer>(
            "Card title",
            TypographyRole::BodyStrong,
            TextRole::Primary,
        );
        let mut tree =
            widget::Tree::new(&widget as &dyn Widget<(), crate::theme::Theme, RecordingRenderer>);
        let renderer = RecordingRenderer;

        let _node = <text::Text<'_, crate::theme::Theme, RecordingRenderer> as Widget<
            (),
            crate::theme::Theme,
            RecordingRenderer,
        >>::layout(&mut widget, &mut tree, &renderer, &layout::Limits::NONE);

        let state = tree.state.downcast_ref::<text::State<RecordingParagraph>>();
        let expected = crate::theme::typography(TypographyRole::BodyStrong);

        assert_eq!(state.raw().font, expected.font);
        assert_eq!(state.raw().size, Pixels(expected.size));
        assert_eq!(
            state.raw().line_height,
            text::LineHeight::Relative(expected.line_height)
        );
    }

    #[derive(Debug, Default)]
    struct RecordingRenderer;

    impl renderer::Renderer for RecordingRenderer {
        fn start_layer(&mut self, _bounds: Rectangle) {}

        fn end_layer(&mut self) {}

        fn start_transformation(&mut self, _transformation: Transformation) {}

        fn end_transformation(&mut self) {}

        fn fill_quad(&mut self, _quad: renderer::Quad, _background: impl Into<Background>) {}

        fn reset(&mut self, _new_bounds: Rectangle) {}

        fn allocate_image(
            &mut self,
            _handle: &iced::advanced::image::Handle,
            _callback: impl FnOnce(Result<iced::advanced::image::Allocation, iced::advanced::image::Error>)
                + Send
                + 'static,
        ) {
        }
    }

    impl advanced_text::Renderer for RecordingRenderer {
        type Font = Font;
        type Paragraph = RecordingParagraph;
        type Editor = ();

        const ICON_FONT: Font = Font::DEFAULT;
        const CHECKMARK_ICON: char = '0';
        const ARROW_DOWN_ICON: char = '0';
        const SCROLL_UP_ICON: char = '0';
        const SCROLL_DOWN_ICON: char = '0';
        const SCROLL_LEFT_ICON: char = '0';
        const SCROLL_RIGHT_ICON: char = '0';
        const ICED_LOGO: char = '0';

        fn default_font(&self) -> Self::Font {
            Font::default()
        }

        fn default_size(&self) -> Pixels {
            Pixels(16.0)
        }

        fn fill_paragraph(
            &mut self,
            _paragraph: &Self::Paragraph,
            _position: Point,
            _color: Color,
            _clip_bounds: Rectangle,
        ) {
        }

        fn fill_editor(
            &mut self,
            _editor: &Self::Editor,
            _position: Point,
            _color: Color,
            _clip_bounds: Rectangle,
        ) {
        }

        fn fill_text(
            &mut self,
            _paragraph: advanced_text::Text<String, Self::Font>,
            _position: Point,
            _color: Color,
            _clip_bounds: Rectangle,
        ) {
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct RecordingParagraph {
        line_height: text::LineHeight,
        size: Pixels,
        font: Font,
        align_x: advanced_text::Alignment,
        align_y: alignment::Vertical,
        wrapping: advanced_text::Wrapping,
        shaping: advanced_text::Shaping,
        bounds: Size,
    }

    impl Default for RecordingParagraph {
        fn default() -> Self {
            Self {
                line_height: text::LineHeight::default(),
                size: Pixels(16.0),
                font: Font::DEFAULT,
                align_x: advanced_text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                wrapping: advanced_text::Wrapping::default(),
                shaping: advanced_text::Shaping::Basic,
                bounds: Size::ZERO,
            }
        }
    }

    impl advanced_text::Paragraph for RecordingParagraph {
        type Font = Font;

        fn with_text(text: advanced_text::Text<&str, Self::Font>) -> Self {
            Self {
                line_height: text.line_height,
                size: text.size,
                font: text.font,
                align_x: text.align_x,
                align_y: text.align_y,
                wrapping: text.wrapping,
                shaping: text.shaping,
                bounds: text.bounds,
            }
        }

        fn with_spans<Link>(
            text: advanced_text::Text<&[advanced_text::Span<'_, Link, Self::Font>], Self::Font>,
        ) -> Self {
            Self {
                line_height: text.line_height,
                size: text.size,
                font: text.font,
                align_x: text.align_x,
                align_y: text.align_y,
                wrapping: text.wrapping,
                shaping: text.shaping,
                bounds: text.bounds,
            }
        }

        fn resize(&mut self, new_bounds: Size) {
            self.bounds = new_bounds;
        }

        fn compare(&self, text: advanced_text::Text<(), Self::Font>) -> advanced_text::Difference {
            if self.line_height == text.line_height && self.size == text.size {
                advanced_text::Difference::None
            } else {
                advanced_text::Difference::Shape
            }
        }

        fn size(&self) -> Pixels {
            self.size
        }

        fn font(&self) -> Self::Font {
            self.font
        }

        fn line_height(&self) -> text::LineHeight {
            self.line_height
        }

        fn align_x(&self) -> advanced_text::Alignment {
            self.align_x
        }

        fn align_y(&self) -> alignment::Vertical {
            self.align_y
        }

        fn wrapping(&self) -> advanced_text::Wrapping {
            self.wrapping
        }

        fn shaping(&self) -> advanced_text::Shaping {
            self.shaping
        }

        fn bounds(&self) -> Size {
            self.bounds
        }

        fn min_bounds(&self) -> Size {
            Size::new(1.0, self.line_height.to_absolute(self.size).0)
        }

        fn hit_test(&self, _point: Point) -> Option<advanced_text::Hit> {
            None
        }

        fn hit_span(&self, _point: Point) -> Option<usize> {
            None
        }

        fn span_bounds(&self, _index: usize) -> Vec<Rectangle> {
            Vec::new()
        }

        fn grapheme_position(&self, _line: usize, _index: usize) -> Option<Point> {
            None
        }
    }
}

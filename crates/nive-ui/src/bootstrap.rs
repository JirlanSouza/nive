use iced::{
    border::Radius,
    time::Duration,
    widget::{column, container, row, stack, svg, text as iced_text, Space},
    Alignment, Background, Border, Color, ContentFit, Length, Padding, Shadow, Vector,
};

use crate::{
    theme::{
        self, surface as theme_surface, GapRole, SurfaceRole, TextRole, ToneRole, TypographyRole,
    },
    widgets::{
        text, AnimatedVisual, Animation, AnimationFrame, ErrorDetailsDialog, ErrorFeedbackAction,
        ErrorFeedbackActionRow, StaggeredPulse,
    },
    Element,
};

const CONTENT_TOP_FILL_PORTION: u16 = 9;
const CONTENT_BOTTOM_FILL_PORTION: u16 = 11;
const ERROR_PANEL_WIDTH: f32 = 520.0;
const BRAND_MARK_SIDE: f32 = 108.0;
const STATUS_DOT_COUNT: usize = 3;
const STATUS_DOT_SIDE: f32 = 5.0;
const STATUS_DOT_SPACING: f32 = 3.0;
const STATUS_DOTS_HEIGHT: f32 = 13.0;
const STATUS_DOTS_WIDTH: f32 =
    STATUS_DOT_SIDE * STATUS_DOT_COUNT as f32 + STATUS_DOT_SPACING * (STATUS_DOT_COUNT - 1) as f32;
const STATUS_DOTS_ANIMATION_DURATION_MS: u64 = 1200;
const STATUS_DOT_PULSE: StaggeredPulse = StaggeredPulse::new(STATUS_DOT_COUNT)
    .overlap(0.65)
    .rest(0.28);

pub struct BootstrapView<'a, Message> {
    title: &'a str,
    subtitle: Option<&'a str>,
    logo_svg: Option<&'static [u8]>,
    background_svg: Option<&'static [u8]>,
    background_fit: ContentFit,
    background_opacity: f32,
    loading_message: &'a str,
    failure_title: &'a str,
    failure_message: &'a str,
    error: Option<BootstrapError<'a>>,
    on_retry: Message,
    on_show_details: Message,
    on_close_details: Message,
}

#[derive(Debug, Clone, Copy)]
pub struct BootstrapError<'a> {
    pub summary: &'a str,
    pub detail: &'a str,
    pub has_diagnostic_detail: bool,
    pub details_visible: bool,
}

impl<'a, Message> BootstrapView<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(
        title: &'a str,
        loading_message: &'a str,
        failure_title: &'a str,
        failure_message: &'a str,
        on_retry: Message,
        on_show_details: Message,
        on_close_details: Message,
    ) -> Self {
        Self {
            title,
            subtitle: None,
            logo_svg: None,
            background_svg: None,
            background_fit: ContentFit::Cover,
            background_opacity: 1.0,
            loading_message,
            failure_title,
            failure_message,
            error: None,
            on_retry,
            on_show_details,
            on_close_details,
        }
    }

    pub fn subtitle(mut self, subtitle: Option<&'a str>) -> Self {
        self.subtitle = subtitle;
        self
    }

    pub fn logo(mut self, svg: Option<&'static [u8]>) -> Self {
        self.logo_svg = svg;
        self
    }

    pub fn background(
        mut self,
        svg: Option<&'static [u8]>,
        content_fit: ContentFit,
        opacity: f32,
    ) -> Self {
        self.background_svg = svg;
        self.background_fit = content_fit;
        self.background_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn error(mut self, error: Option<BootstrapError<'a>>) -> Self {
        self.error = error;
        self
    }

    pub fn content(&self) -> Element<'a, Message> {
        let content = match self.error {
            Some(error) => self.failure_content(error),
            None => self.loading_content(),
        };

        container(stack![
            self.background_content(),
            column![
                Space::new().height(Length::FillPortion(CONTENT_TOP_FILL_PORTION)),
                content,
                Space::new().height(Length::FillPortion(CONTENT_BOTTOM_FILL_PORTION)),
            ]
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill),
        ])
        .style(theme_surface::style(SurfaceRole::App))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn details_dialog(&self) -> Option<Element<'a, Message>> {
        self.error
            .filter(|error| error.details_visible)
            .map(|error| {
                ErrorDetailsDialog::new(error.detail)
                    .on_close(self.on_close_details.clone())
                    .into()
            })
    }

    fn loading_content(&self) -> Element<'a, Message> {
        column![self.brand_identity(), self.startup_status()]
            .spacing(theme::gap(GapRole::Content))
            .align_x(Alignment::Center)
            .into()
    }

    fn failure_content(&self, error: BootstrapError<'a>) -> Element<'a, Message> {
        let mut actions = vec![ErrorFeedbackAction::primary_retry(
            "Try again",
            self.on_retry.clone(),
        )];
        if error.has_diagnostic_detail {
            actions.push(ErrorFeedbackAction::details(self.on_show_details.clone()));
        }

        let copy = column![
            text::title(self.failure_title).center(),
            text::body(self.failure_message)
                .style(theme::text::style(TextRole::Muted))
                .center(),
        ]
        .spacing(theme::gap(GapRole::Tight))
        .align_x(Alignment::Center)
        .width(Length::Fill);

        let mut body = column![self.brand_mark(), copy]
            .spacing(theme::gap(GapRole::Content))
            .align_x(Alignment::Center)
            .width(Length::Fill);

        if !error.has_diagnostic_detail {
            body = body.push(
                text::body_small(error.summary)
                    .style(theme::text::tone(ToneRole::Danger))
                    .center(),
            );
        }

        body = body.push(ErrorFeedbackActionRow::new(actions));

        container(body)
            .width(Length::Fill)
            .max_width(ERROR_PANEL_WIDTH)
            .padding(Padding::new(theme::gap(GapRole::Section)))
            .into()
    }

    fn brand_identity(&self) -> Element<'a, Message> {
        let mut identity = column![self.brand_mark(), self.wordmark()]
            .spacing(theme::gap(GapRole::Related))
            .align_x(Alignment::Center);

        if let Some(subtitle) = self.subtitle {
            identity = identity.push(
                text::body_small(subtitle)
                    .style(theme::text::style(TextRole::Muted))
                    .center(),
            );
        }

        identity.into()
    }

    fn brand_mark(&self) -> Element<'a, Message> {
        let mark: Element<'a, Message> = self
            .logo_svg
            .map(|logo| {
                svg(svg::Handle::from_memory(logo))
                    .width(Length::Fixed(BRAND_MARK_SIDE))
                    .height(Length::Fixed(BRAND_MARK_SIDE))
                    .content_fit(ContentFit::Contain)
                    .into()
            })
            .unwrap_or_else(|| Space::new().into());

        container(mark)
            .center(Length::Fixed(BRAND_MARK_SIDE))
            .style(brand_mark_style())
            .into()
    }

    fn wordmark(&self) -> Element<'a, Message> {
        text::heading(self.title)
            .size(31)
            .style(theme::text::style(TextRole::Primary))
            .center()
            .into()
    }

    fn startup_status(&self) -> Element<'a, Message> {
        row![
            status_dots(),
            text::body_small(self.loading_message)
                .size(theme::typography(TypographyRole::BodySmall).size + 1.0)
                .style(status_label_style()),
        ]
        .spacing(theme::gap(GapRole::Tight))
        .align_y(Alignment::Center)
        .into()
    }

    fn background_content(&self) -> Element<'a, Message> {
        let Some(background) = self.background_svg else {
            return Space::new().into();
        };
        let opacity = self.background_opacity;

        stack![
            svg(svg::Handle::from_memory(background))
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(self.background_fit),
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |theme: &crate::theme::Theme| container::Style {
                    background: Some(Background::Color(
                        theme
                            .surface(SurfaceRole::App)
                            .background
                            .scale_alpha(1.0 - opacity),
                    )),
                    ..container::Style::default()
                }),
        ]
        .into()
    }
}

fn status_dots<'a, Message>() -> Element<'a, Message>
where
    Message: 'a,
{
    AnimatedVisual::new(|frame| -> Element<'a, Message> {
        let dots = (0..STATUS_DOT_COUNT)
            .map(|offset| status_dot(frame, offset))
            .collect::<Vec<_>>();

        container(
            row(dots)
                .spacing(STATUS_DOT_SPACING)
                .align_y(Alignment::Center),
        )
        .width(Length::Fixed(STATUS_DOTS_WIDTH))
        .height(Length::Fixed(STATUS_DOTS_HEIGHT))
        .align_y(Alignment::Center)
        .into()
    })
    .animation(Animation::linear(Duration::from_millis(STATUS_DOTS_ANIMATION_DURATION_MS)).repeat())
    .into()
}

fn status_dot<'a, Message>(frame: AnimationFrame, offset: usize) -> Element<'a, Message>
where
    Message: 'a,
{
    container(Space::new())
        .width(Length::Fixed(STATUS_DOT_SIDE))
        .height(Length::Fixed(STATUS_DOT_SIDE))
        .style(status_dot_style(STATUS_DOT_PULSE.activity(frame, offset)))
        .into()
}

fn brand_mark_style() -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let accent = startup_accent(theme);
        let glow_alpha = if theme.is_dark() { 0.16 } else { 0.1 };

        container::Style {
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(28.0),
            },
            shadow: Shadow {
                color: accent.scale_alpha(glow_alpha),
                offset: Vector::new(0.0, 5.0),
                blur_radius: 18.0,
            },
            ..container::Style::default()
        }
    }
}

fn status_dot_style(activity: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let accent = startup_accent(theme);
        let alpha = 0.34 + activity * 0.58;

        container::Style {
            background: Some(Background::Color(accent.scale_alpha(alpha))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(STATUS_DOT_SIDE / 2.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

fn status_label_style() -> impl Fn(&crate::theme::Theme) -> iced_text::Style {
    move |theme| iced_text::Style {
        color: Some(
            blend(
                theme.text(TextRole::Secondary).color,
                theme.text(TextRole::Primary).color,
                0.5,
            )
            .scale_alpha(0.92),
        ),
    }
}

fn startup_accent(theme: &crate::theme::Theme) -> Color {
    let accent = theme.tone(ToneRole::Primary).color;
    let lift = if theme.is_dark() { 0.28 } else { 0.04 };

    blend(accent, Color::WHITE, lift)
}

fn blend(from: Color, to: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);

    Color {
        r: from.r + (to.r - from.r) * amount,
        g: from.g + (to.g - from.g) * amount,
        b: from.b + (to.b - from.b) * amount,
        a: from.a + (to.a - from.a) * amount,
    }
}

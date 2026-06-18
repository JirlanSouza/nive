use iced::Task;

use crate::UserFacingResult;

#[derive(Debug, Clone, PartialEq)]
pub struct BrandContent {
    title: String,
    subtitle: Option<String>,
    logo: Option<&'static [u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackgroundFit {
    Contain,
    #[default]
    Cover,
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackgroundPosition {
    #[default]
    Center,
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplashBackground {
    svg: &'static [u8],
    opacity: f32,
    fit: BackgroundFit,
    position: BackgroundPosition,
}

pub struct BootstrapSpec<B> {
    task: Task<UserFacingResult<B>>,
    brand: Option<BrandContent>,
    background: Option<SplashBackground>,
    loading_message: String,
    failure_title: String,
    failure_message: String,
}

impl BrandContent {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            logo: None,
        }
    }

    pub fn logo(mut self, svg: &'static [u8]) -> Self {
        self.logo = Some(svg);
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn subtitle_text(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub fn logo_svg(&self) -> Option<&'static [u8]> {
        self.logo
    }
}

impl SplashBackground {
    pub fn svg(svg: &'static [u8]) -> Self {
        Self {
            svg,
            opacity: 1.0,
            fit: BackgroundFit::Cover,
            position: BackgroundPosition::Center,
        }
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn fit(mut self, fit: BackgroundFit) -> Self {
        self.fit = fit;
        self
    }

    pub fn position(mut self, position: BackgroundPosition) -> Self {
        self.position = position;
        self
    }

    pub fn svg_bytes(&self) -> &'static [u8] {
        self.svg
    }

    pub fn opacity_value(&self) -> f32 {
        self.opacity
    }

    pub fn fit_mode(&self) -> BackgroundFit {
        self.fit
    }

    pub fn alignment(&self) -> BackgroundPosition {
        self.position
    }
}

impl<B> BootstrapSpec<B> {
    pub fn new(task: Task<UserFacingResult<B>>) -> Self {
        Self {
            task,
            brand: None,
            background: None,
            loading_message: String::from("Preparing application"),
            failure_title: String::from("Application couldn't start"),
            failure_message: String::from(
                "A startup issue prevented the application from opening.",
            ),
        }
    }

    pub fn brand(mut self, brand: BrandContent) -> Self {
        self.brand = Some(brand);
        self
    }

    pub fn background(mut self, background: SplashBackground) -> Self {
        self.background = Some(background);
        self
    }

    pub fn loading_message(mut self, message: impl Into<String>) -> Self {
        self.loading_message = message.into();
        self
    }

    pub fn failure_title(mut self, title: impl Into<String>) -> Self {
        self.failure_title = title.into();
        self
    }

    pub fn failure_message(mut self, message: impl Into<String>) -> Self {
        self.failure_message = message.into();
        self
    }

    pub fn task(&self) -> &Task<UserFacingResult<B>> {
        &self.task
    }

    pub fn brand_content(&self) -> Option<&BrandContent> {
        self.brand.as_ref()
    }

    pub fn splash_background(&self) -> Option<&SplashBackground> {
        self.background.as_ref()
    }

    pub fn loading_text(&self) -> &str {
        self.loading_message.as_str()
    }

    pub fn failure_heading(&self) -> &str {
        self.failure_title.as_str()
    }

    pub fn failure_text(&self) -> &str {
        self.failure_message.as_str()
    }
}

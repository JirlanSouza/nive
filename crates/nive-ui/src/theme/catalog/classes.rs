use iced::{
    overlay::menu,
    widget::{
        button, checkbox, container, pick_list, progress_bar, rule, scrollable, text, text_input,
        toggler,
    },
};

use super::super::{TextRole, Theme, ToneRole};

#[derive(Default)]
pub enum ButtonClass<'a> {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
    Embedded,
    Custom(button::StyleFn<'a, Theme>),
}

#[derive(Default)]
pub enum CheckboxClass<'a> {
    #[default]
    Standard,
    Custom(checkbox::StyleFn<'a, Theme>),
}

#[derive(Default)]
pub enum ContainerClass<'a> {
    #[default]
    Transparent,
    Custom(container::StyleFn<'a, Theme>),
}

#[derive(Default)]
pub enum TextClass<'a> {
    #[default]
    Default,
    Role(TextRole),
    Tone(ToneRole),
    OnTone(ToneRole),
    Custom(text::StyleFn<'a, Theme>),
}

pub enum TextInputClass<'a> {
    Standard { validation: FieldValidation },
    Embedded { validation: FieldValidation },
    Custom(text_input::StyleFn<'a, Theme>),
}

#[derive(Default)]
pub enum TogglerClass<'a> {
    #[default]
    Standard,
    Custom(toggler::StyleFn<'a, Theme>),
}

#[derive(Default)]
pub enum RuleClass<'a> {
    #[default]
    Default,
    Custom(rule::StyleFn<'a, Theme>),
}

#[derive(Default)]
pub enum ProgressBarClass<'a> {
    #[default]
    Default,
    Custom(progress_bar::StyleFn<'a, Theme>),
}

#[derive(Default)]
pub enum ScrollableClass<'a> {
    #[default]
    Default,
    Custom(scrollable::StyleFn<'a, Theme>),
}

#[derive(Default)]
pub enum MenuClass<'a> {
    #[default]
    Default,
    Custom(menu::StyleFn<'a, Theme>),
}

#[derive(Default)]
pub enum PickListClass<'a> {
    #[default]
    Default,
    Custom(pick_list::StyleFn<'a, Theme>),
}

impl<'a> Default for TextInputClass<'a> {
    fn default() -> Self {
        Self::Standard {
            validation: FieldValidation::Valid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldValidation {
    #[default]
    Valid,
    Invalid,
}

impl<'a> From<button::StyleFn<'a, Theme>> for ButtonClass<'a> {
    fn from(style: button::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<checkbox::StyleFn<'a, Theme>> for CheckboxClass<'a> {
    fn from(style: checkbox::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<container::StyleFn<'a, Theme>> for ContainerClass<'a> {
    fn from(style: container::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<text::StyleFn<'a, Theme>> for TextClass<'a> {
    fn from(style: text::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<text_input::StyleFn<'a, Theme>> for TextInputClass<'a> {
    fn from(style: text_input::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<toggler::StyleFn<'a, Theme>> for TogglerClass<'a> {
    fn from(style: toggler::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<rule::StyleFn<'a, Theme>> for RuleClass<'a> {
    fn from(style: rule::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<progress_bar::StyleFn<'a, Theme>> for ProgressBarClass<'a> {
    fn from(style: progress_bar::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<scrollable::StyleFn<'a, Theme>> for ScrollableClass<'a> {
    fn from(style: scrollable::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<menu::StyleFn<'a, Theme>> for MenuClass<'a> {
    fn from(style: menu::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

impl<'a> From<pick_list::StyleFn<'a, Theme>> for PickListClass<'a> {
    fn from(style: pick_list::StyleFn<'a, Theme>) -> Self {
        Self::Custom(style)
    }
}

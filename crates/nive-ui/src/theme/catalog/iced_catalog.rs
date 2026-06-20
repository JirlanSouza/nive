use iced::{
    overlay::menu,
    widget::{
        button, checkbox, container, pick_list, progress_bar, rule, scrollable, svg, text,
        text_input, toggler,
    },
};

use super::button::{
    destructive_button, embedded_button, ghost_button, link_button, outline_button, primary_button,
    secondary_button,
};
use super::classes::{
    ButtonClass, CheckboxClass, ContainerClass, MenuClass, PickListClass, ProgressBarClass,
    RuleClass, ScrollableClass, TextClass, TextInputClass, TogglerClass,
};
use super::controls::{
    default_checkbox, default_toggler, embedded_text_input, standard_text_input,
};
use super::misc::{
    default_menu, default_pick_list, default_progress_bar, default_rule, default_scrollable,
};
use crate::theme::Theme;

impl button::Catalog for Theme {
    type Class<'a> = ButtonClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        ButtonClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        match class {
            ButtonClass::Primary => primary_button(self, status),
            ButtonClass::Secondary => secondary_button(self, status),
            ButtonClass::Outline => outline_button(self, status),
            ButtonClass::Ghost => ghost_button(self, status),
            ButtonClass::Destructive => destructive_button(self, status),
            ButtonClass::Link => link_button(self, status),
            ButtonClass::Embedded => embedded_button(self, status),
            ButtonClass::Custom(style) => style(self, status),
        }
    }
}

impl checkbox::Catalog for Theme {
    type Class<'a> = CheckboxClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        CheckboxClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: checkbox::Status) -> checkbox::Style {
        match class {
            CheckboxClass::Standard => default_checkbox(self, status),
            CheckboxClass::Custom(style) => style(self, status),
        }
    }
}

impl container::Catalog for Theme {
    type Class<'a> = ContainerClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        ContainerClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        match class {
            ContainerClass::Transparent => container::Style::default(),
            ContainerClass::Custom(style) => style(self),
        }
    }
}

impl text_input::Catalog for Theme {
    type Class<'a> = TextInputClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        TextInputClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: text_input::Status) -> text_input::Style {
        match class {
            TextInputClass::Standard { validation } => {
                standard_text_input(self, *validation, status)
            }
            TextInputClass::Embedded { validation } => {
                embedded_text_input(self, *validation, status)
            }
            TextInputClass::Custom(style) => style(self, status),
        }
    }
}

impl toggler::Catalog for Theme {
    type Class<'a> = TogglerClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        TogglerClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: toggler::Status) -> toggler::Style {
        match class {
            TogglerClass::Standard => default_toggler(self, status),
            TogglerClass::Custom(style) => style(self, status),
        }
    }
}

impl text::Catalog for Theme {
    type Class<'a> = TextClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        TextClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> text::Style {
        match class {
            TextClass::Default => text::Style::default(),
            TextClass::Role(role) => text::Style {
                color: Some(self.text(*role).color),
            },
            TextClass::Tone(role) => text::Style {
                color: Some(self.tone(*role).color),
            },
            TextClass::OnTone(role) => text::Style {
                color: Some(self.tone(*role).on_color),
            },
            TextClass::Custom(style) => style(self),
        }
    }
}

impl svg::Catalog for Theme {
    type Class<'a> = svg::StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme, _status| svg::Style::default())
    }

    fn style(&self, class: &Self::Class<'_>, status: svg::Status) -> svg::Style {
        class(self, status)
    }
}

impl rule::Catalog for Theme {
    type Class<'a> = RuleClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        RuleClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> rule::Style {
        match class {
            RuleClass::Default => default_rule(self),
            RuleClass::Custom(style) => style(self),
        }
    }
}

impl progress_bar::Catalog for Theme {
    type Class<'a> = ProgressBarClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        ProgressBarClass::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> progress_bar::Style {
        match class {
            ProgressBarClass::Default => default_progress_bar(self),
            ProgressBarClass::Custom(style) => style(self),
        }
    }
}

impl scrollable::Catalog for Theme {
    type Class<'a> = ScrollableClass<'a>;

    fn default<'a>() -> Self::Class<'a> {
        ScrollableClass::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: scrollable::Status) -> scrollable::Style {
        match class {
            ScrollableClass::Default => default_scrollable(self, status),
            ScrollableClass::Custom(style) => style(self, status),
        }
    }
}

impl menu::Catalog for Theme {
    type Class<'a> = MenuClass<'a>;

    fn default<'a>() -> <Self as menu::Catalog>::Class<'a> {
        MenuClass::default()
    }

    fn style(&self, class: &<Self as menu::Catalog>::Class<'_>) -> menu::Style {
        match class {
            MenuClass::Default => default_menu(self),
            MenuClass::Custom(style) => style(self),
        }
    }
}

impl pick_list::Catalog for Theme {
    type Class<'a> = PickListClass<'a>;

    fn default<'a>() -> <Self as pick_list::Catalog>::Class<'a> {
        PickListClass::default()
    }

    fn style(
        &self,
        class: &<Self as pick_list::Catalog>::Class<'_>,
        status: pick_list::Status,
    ) -> pick_list::Style {
        match class {
            PickListClass::Default => default_pick_list(self, status),
            PickListClass::Custom(style) => style(self, status),
        }
    }
}

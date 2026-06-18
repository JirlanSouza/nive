use nive_ui::prelude::*;

#[test]
fn prelude_exposes_common_ui_contracts() {
    let _: Element<'_, ()> = text("Nive").into();
    let _: Theme = theme::active();
    let _: ThemePreference = ThemePreference::System;
}

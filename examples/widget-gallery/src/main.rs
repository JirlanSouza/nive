mod app;
mod catalog;
mod fixtures;
mod layout;
mod pages;

fn main() -> nive::Result {
    nive::run_with_devtools::<app::WidgetGallery>()
}

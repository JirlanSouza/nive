mod app;
mod catalog;
#[cfg(feature = "devtools")]
mod fixtures;
mod icons;
mod layout;
mod pages;

fn main() -> nive::Result {
    #[cfg(feature = "devtools")]
    {
        nive::run_with_devtools::<app::WidgetGallery>()
    }

    #[cfg(not(feature = "devtools"))]
    {
        nive::run::<app::WidgetGallery>()
    }
}

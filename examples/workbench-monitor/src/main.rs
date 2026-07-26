mod app;
mod icons;
mod sim;

fn main() -> nive::Result {
    nive::run::<app::WorkbenchMonitor>()
}

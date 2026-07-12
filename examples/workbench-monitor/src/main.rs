mod app;
mod sim;

fn main() -> nive::Result {
    nive::run::<app::WorkbenchMonitor>()
}

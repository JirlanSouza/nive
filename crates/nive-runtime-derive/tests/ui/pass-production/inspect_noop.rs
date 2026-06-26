// Pass without the devtools feature: derive expands to nothing, so fields do
// not need to implement the runtime Inspect trait.
use nive_runtime_derive::Inspect;

struct NotInspect;

#[derive(Inspect)]
struct AppState {
    #[inspect(default)]
    data: NotInspect,
}

fn main() {}

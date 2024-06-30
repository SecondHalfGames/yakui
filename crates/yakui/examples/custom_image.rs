use yakui::image;

use bootstrap::ExampleState;

pub fn run(state: &mut ExampleState) {
    image(state.custom.unwrap(), [512.0, 512.0]);
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}

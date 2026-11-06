use bootstrap::ExampleState;
use yakui::{Vec2, image};

pub fn run(state: &mut ExampleState) {
    image(state.monkey_transparent, Vec2::splat(100.0));
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}

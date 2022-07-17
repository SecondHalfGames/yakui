use yakui::Vec2;

use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    yakui::image(state.monkey, Vec2::new(400.0, 400.0));
}

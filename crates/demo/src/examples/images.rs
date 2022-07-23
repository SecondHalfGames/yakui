use yakui::{image, Vec2};

use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    image(state.monkey, Vec2::new(400.0, 400.0));
}

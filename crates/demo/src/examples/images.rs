use yakui::{image, ninepatch, pad, widgets::Pad, Vec2};

use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    pad(Pad::all(20.0), || {
        ninepatch(state.brown_inlay, Pad::all(15.0), 4.0, || {
            image(state.monkey, Vec2::new(400.0, 400.0));
        });
    });
}

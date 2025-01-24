use yakui::center;
use yakui::{image, nineslice, pad, widgets::Pad, Vec2};

use bootstrap::ExampleState;

pub fn run(state: &mut ExampleState) {
    pad(Pad::all(20.0), || {
        nineslice(state.brown_inlay, Pad::all(15.0), 9.0, || {
            center(|| {
                image(state.monkey, Vec2::splat(800.0));
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}

use yakui::widgets::Image;
use yakui::{center, Color};
use yakui::{nineslice, widgets::Pad, Vec2};

use bootstrap::ExampleState;

pub fn run(state: &mut ExampleState) {
    let shade = ((state.time.sin() * 0.5 + 0.5) * 255.0).round() as u8;
    let tint = Color::greyscale(shade);

    Pad::all(20.0).show(|| {
        nineslice(state.brown_inlay, Pad::all(15.0), 9.0, || {
            center(|| {
                Image::new(state.monkey, Vec2::splat(800.0))
                    .color(tint)
                    .show();
            });
        });
    });
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}

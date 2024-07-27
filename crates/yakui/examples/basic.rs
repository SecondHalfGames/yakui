use yakui::{Color, Vec2};
use yakui_widgets::colored_box;

pub fn run() {
    colored_box(Color::RED, Vec2::splat(100.0));
}

fn main() {
    bootstrap::start(run as fn());
}

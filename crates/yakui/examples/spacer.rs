use yakui::{colored_box, row, Color};
use yakui_widgets::spacer;

pub fn run() {
    row(|| {
        colored_box(Color::RED, [100.0, 100.0]);
        spacer(2);
        colored_box(Color::GREEN, [100.0, 100.0]);
        spacer(1);
        colored_box(Color::BLUE, [100.0, 100.0]);
    });
}

fn main() {
    bootstrap::start(run as fn());
}

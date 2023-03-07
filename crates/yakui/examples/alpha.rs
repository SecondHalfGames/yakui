use yakui::widgets::Pad;
use yakui::{colored_box, colored_box_container, pad, Color};

fn run() {
    colored_box_container(Color::rgba(255, 0, 0, 127), || {
        pad(Pad::all(50.0), || {
            colored_box(Color::rgba(0, 0, 255, 127), [50.0, 50.0]);
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}

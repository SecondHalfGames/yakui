use yakui::{colored_box, expanded, row, Color};

pub fn run() {
    row(|| {
        colored_box(Color::RED, [100.0, 100.0]);
        expanded(|| {
            colored_box(Color::GREEN, [100.0, 100.0]);
        });
        colored_box(Color::BLUE, [100.0, 100.0]);
    });
}

fn main() {
    bootstrap::start(run as fn());
}

use yakui::{colored_box, expanded, row, Color};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    row(|| {
        colored_box(Color::RED, [100.0, 100.0]);
        expanded(|| {
            colored_box(Color::GREEN, [100.0, 100.0]);
        });
        colored_box(Color::BLUE, [100.0, 100.0]);
    });
}

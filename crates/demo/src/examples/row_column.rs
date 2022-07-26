use yakui::{colored_box, column, row, Color};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    column(|| {
        colored_box(Color::RED, [100.0, 100.0]);
        row(|| {
            colored_box(Color::GREEN, [100.0, 100.0]);
            colored_box(Color::REBECCA_PURPLE, [100.0, 100.0]);
            colored_box(Color::YELLOW, [100.0, 100.0]);
        });
        colored_box(Color::BLUE, [100.0, 100.0]);
    });
}

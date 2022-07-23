use yakui::{colored_box, column, row, Color3};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    column(|| {
        colored_box(Color3::RED, [100.0, 100.0]);
        row(|| {
            colored_box(Color3::GREEN, [100.0, 100.0]);
            colored_box(Color3::REBECCA_PURPLE, [100.0, 100.0]);
            colored_box(Color3::YELLOW, [100.0, 100.0]);
        });
        colored_box(Color3::BLUE, [100.0, 100.0]);
    });
}

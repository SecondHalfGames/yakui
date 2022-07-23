use yakui::{colored_box, expanded, row, Color3};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    row(|| {
        colored_box(Color3::RED, [100.0, 100.0]);
        expanded(|| {
            colored_box(Color3::GREEN, [100.0, 100.0]);
        });
        colored_box(Color3::BLUE, [100.0, 100.0]);
    });
}

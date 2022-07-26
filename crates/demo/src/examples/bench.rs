use yakui::{colored_box, column, row, Color};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    let colors = [Color::hex(0x3a2c32), Color::hex(0x222a29)];

    row(|| {
        for x in 0..100 {
            column(|| {
                for y in 0..100 {
                    let color = colors[(x + y) % colors.len()];
                    colored_box(color, [10.0, 10.0]);
                }
            });
        }
    });
}

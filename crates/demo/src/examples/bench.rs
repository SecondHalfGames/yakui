use yakui::{colored_box, column, row, Color3};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    let colors = [Color3::RED, Color3::GREEN, Color3::BLUE];

    row(|| {
        for x in 0..100 {
            column(|| {
                for y in 0..100 {
                    let color = colors[(x + y) % colors.len()];

                    let w = 2.0 + 3.0 + (x / 2) as f32;
                    let h = 2.0 + 3.0 + (y / 2) as f32;
                    colored_box(color, [w, h]);
                }
            });
        }
    });
}

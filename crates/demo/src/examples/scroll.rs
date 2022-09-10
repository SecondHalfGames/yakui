use yakui::{button, colored_box_container, column, scroll_vertical, Color};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    scroll_vertical(|| {
        colored_box_container(Color::hex(0x888888), || {
            column(|| {
                for i in 0..100 {
                    button(format!("List item {i}"));
                }
            });
        });
    });
}

use yakui::{button, column, scroll_vertical};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    scroll_vertical(|| {
        column(|| {
            for i in 0..100 {
                button(format!("List item {i}"));
            }
        });
    });
}

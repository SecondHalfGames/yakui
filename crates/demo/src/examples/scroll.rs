use yakui::widgets::Pad;
use yakui::{button, center, column, expanded, pad, scroll_vertical};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    center(|| {
        pad(Pad::all(50.0), || {
            scroll_vertical(|| {
                column(|| {
                    for i in 0..100 {
                        expanded(|| {
                            if button(format!("List item {i}")).clicked {
                                println!("Clicked button {i}");
                            }
                        });
                    }
                });
            });
        });
    });
}

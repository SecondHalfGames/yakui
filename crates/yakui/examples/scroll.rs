use yakui::widgets::Pad;
use yakui::{button, center, column, expanded, pad, scroll_vertical};

pub fn run() {
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

fn main() {
    bootstrap::start(run as fn());
}

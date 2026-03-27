use yakui::widgets::{List, Pad};
use yakui::{button, center, colored_box_container, column, label, opaque, Color};

pub fn run() {
    frame(|| {
        opaque(|| {
            colored_box_container(Color::BLACK, || {
                Pad::all(20.0).show(|| {
                    column(|| {
                        label("Clicking inside this box will sink the input.");

                        if button("Test Button").clicked {
                            println!("First button clicked");
                        }
                    });
                });
            });
        });

        colored_box_container(Color::BLACK, || {
            Pad::all(20.0).show(|| {
                column(|| {
                    label("Clicking here will let the input through.");

                    if button("Test Button").clicked {
                        println!("Second button clicked");
                    }
                });
            });
        });
    });
}

fn frame(children: impl FnOnce()) {
    center(|| {
        Pad::all(16.0).show(|| {
            List::column().item_spacing(8.0).show(children);
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}

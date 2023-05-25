//! This example shows yakui's progress in having the primitives needed to
//! implement a dropdown or combo box widget.

#![allow(clippy::collapsible_if)]

use yakui::widgets::Layer;
use yakui::{align, button, column, label, reflow, use_state, widgets::Pad, Alignment, Dim2};

pub fn run() {
    let open = use_state(|| false);
    let options = ["Hello", "World", "Foobar"];
    let selected = use_state(|| 0);

    align(Alignment::TOP_LEFT, || {
        column(|| {
            label("First Widget");

            column(|| {
                if button(options[selected.get()]).clicked {
                    open.modify(|x| !x);
                }

                if open.get() {
                    Pad::ZERO.show(|| {
                        Layer::new().show(|| {
                            reflow(Alignment::BOTTOM_LEFT, Dim2::ZERO, || {
                                column(|| {
                                    let current = selected.get();
                                    for (i, option) in options.iter().enumerate() {
                                        if i != current {
                                            if button(*option).clicked {
                                                selected.set(i);
                                                open.set(false);
                                            }
                                        }
                                    }
                                });
                            });
                        });
                    });
                }
            });

            label("Last Widget");
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}

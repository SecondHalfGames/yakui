use yakui::widgets::{List, Panel};
use yakui::{button, expanded, label, row, textbox, CrossAxisAlignment};

use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    row(|| {
        let panel = Panel::side();
        panel.show(|| {
            let mut column = List::column();
            column.cross_axis_alignment = CrossAxisAlignment::Start;

            column.show(|| {
                row(|| {
                    expanded(|| {
                        label("Label Label");
                    });
                    button("Button!");
                });

                row(|| {
                    label("More labels!");
                    button("Buttons!!!");
                });

                row(|| {
                    label("Input");
                    expanded(|| {
                        if let Some(new_name) = textbox(&state.name).text.as_ref() {
                            state.name = new_name.clone();
                        }
                    });
                });

                row(|| {
                    expanded(|| {
                        button("Wide Button!");
                    });
                });
            });
        });
    });
}

use yakui::widgets::{List, Panel};
use yakui::{button, expanded, label, row, CrossAxisAlignment};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
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
                    expanded(|| {
                        button("Wide Button!");
                    });
                });
            });
        });
    });
}

use yakui::widgets::List;
use yakui::CrossAxisAlignment;

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    yakui::row(|| {
        let panel = yakui::widgets::Panel::side();
        panel.show(|| {
            let mut column = List::column();
            column.cross_axis_alignment = CrossAxisAlignment::Start;

            column.show(|| {
                yakui::row(|| {
                    yakui::expanded(|| {
                        yakui::label("Label Label");
                    });
                    yakui::button("Button!");
                });

                yakui::row(|| {
                    yakui::label("More labels!");
                    yakui::button("Buttons!!!");
                });

                yakui::row(|| {
                    yakui::button("Wide Button!");
                });
            });
        });
    });
}

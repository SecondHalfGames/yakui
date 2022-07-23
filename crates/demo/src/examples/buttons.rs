use yakui::widgets::List;
use yakui::{button, row, CrossAxisAlignment};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    row(|| {
        button("Not stretched");
        let mut col = List::column();
        col.cross_axis_alignment = CrossAxisAlignment::Stretch;
        col.show(|| {
            button("Button 1");
            button("Button 2");
            button("Button 3");
        });
    });
}

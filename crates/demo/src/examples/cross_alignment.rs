use yakui::widgets::List;
use yakui::{colored_box, expanded, label, Color3, CrossAxisAlignment};

use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    let alignments = [
        CrossAxisAlignment::Start,
        CrossAxisAlignment::Center,
        CrossAxisAlignment::End,
        CrossAxisAlignment::Stretch,
    ];

    let index = (state.time.floor() as usize) % alignments.len();
    let alignment = alignments[index];

    let mut row = List::row();
    row.cross_axis_alignment = alignment;
    row.show(|| {
        colored_box(Color3::RED, [100.0, 100.0]);
        expanded(|| {
            colored_box(Color3::GREEN, [100.0, 100.0]);
        });
        colored_box(Color3::BLUE, [100.0, 100.0]);
        label(format!("CrossAxisAlignment::{alignment:?}"));
    });
}

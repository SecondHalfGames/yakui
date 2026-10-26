use yakui::widgets::List;
use yakui::{colored_box, expanded, label, Color, CrossAxisAlignment};

use bootstrap::ExampleState;

pub fn run(state: &mut ExampleState) {
    let alignments = [
        CrossAxisAlignment::Start,
        CrossAxisAlignment::Center,
        CrossAxisAlignment::End,
        CrossAxisAlignment::Stretch,
    ];

    let index = (state.time.floor() as usize) % alignments.len();
    let alignment = alignments[index];

    List::row().cross_axis_alignment(alignment).show(|| {
        colored_box(Color::RED, [100.0, 100.0]);
        expanded(|| {
            colored_box(Color::GREEN, [100.0, 100.0]);
        });
        colored_box(Color::BLUE, [100.0, 100.0]);
        label(format!("CrossAxisAlignment::{alignment:?}"));
    });
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}

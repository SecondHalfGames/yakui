use yakui::widgets::List;
use yakui::{colored_box, label, Color, CrossAxisAlignment, MainAxisAlignment};

use bootstrap::ExampleState;

pub fn run(state: &mut ExampleState) {
    let alignments = [
        MainAxisAlignment::Start,
        MainAxisAlignment::Center,
        MainAxisAlignment::End,
        MainAxisAlignment::SpaceBetween,
        MainAxisAlignment::SpaceAround,
        MainAxisAlignment::SpaceEvenly,
    ];

    let index = (state.time.floor() as usize) % alignments.len();
    let alignment = alignments[index];

    List::row()
        .main_axis_alignment(alignment)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .show(|| {
            colored_box(Color::RED, [100.0, 100.0]);
            label(format!("MainAxisAlignment::{alignment:?}"));
            colored_box(Color::BLUE, [100.0, 100.0]);
        });
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}

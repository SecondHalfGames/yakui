use yakui::{align, colored_box, Alignment, Color3};

use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    const ALIGNMENTS: &[Alignment] = &[
        Alignment::TOP_LEFT,
        Alignment::TOP_CENTER,
        Alignment::TOP_RIGHT,
        Alignment::CENTER_LEFT,
        Alignment::CENTER,
        Alignment::CENTER_RIGHT,
        Alignment::BOTTOM_LEFT,
        Alignment::BOTTOM_CENTER,
        Alignment::BOTTOM_RIGHT,
    ];

    let index = (state.time as usize) % ALIGNMENTS.len();
    let alignment = ALIGNMENTS[index];

    align(alignment, || {
        colored_box(Color3::REBECCA_PURPLE, [100.0, 100.0]);
    });
}

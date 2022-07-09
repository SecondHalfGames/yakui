use yakui::{Alignment, Button};

pub fn app(time: f32) {
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

    let index = (time as usize) % ALIGNMENTS.len();
    let alignment = ALIGNMENTS[index];

    yakui::align(alignment, || {
        yakui::button(Button::styled([100.0, 100.0]));
    });
}

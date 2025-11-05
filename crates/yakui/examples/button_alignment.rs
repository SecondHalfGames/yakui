use bootstrap::ExampleState;
use yakui::style::TextAlignment;
use yakui::widgets::Button;

pub fn run(state: &mut ExampleState) {
    const ALIGNMENTS: &[TextAlignment] = &[
        TextAlignment::Start,
        TextAlignment::Center,
        TextAlignment::End,
    ];

    let index = (state.time as usize) % ALIGNMENTS.len();
    let alignment = ALIGNMENTS[index];

    let mut button = Button::styled("X X X X X");
    button.style.text.font_size = 60.0;
    button.hover_style.text.font_size = 60.0;
    button.down_style.text.font_size = 60.0;
    button.style.text.align = alignment;
    button.hover_style.text.align = alignment;
    button.down_style.text.align = alignment;
    button.show();
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}

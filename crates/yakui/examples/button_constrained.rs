use yakui::style::TextAlignment;
use yakui::widgets::{Button, List};
use yakui::{constrained, Constraints, CrossAxisAlignment, Vec2};

pub fn run() {
    const ALIGNMENTS: &[TextAlignment] = &[
        TextAlignment::Start,
        TextAlignment::Center,
        TextAlignment::End,
    ];

    List::row()
        .cross_axis_alignment(CrossAxisAlignment::End)
        .item_spacing(16.0)
        .show(|| {
            for &alignment in ALIGNMENTS {
                menu_button(alignment);
            }
        });
}

fn menu_button(alignment: TextAlignment) {
    let constraints = Constraints {
        min: Vec2::new(120.0, 0.0),
        max: Vec2::new(f32::INFINITY, f32::INFINITY),
    };

    constrained(constraints, || {
        let mut button = Button::styled(format!("Foo ({alignment:?})"));
        button.style.text.font_size = 12.0;
        button.style.text.align = alignment;
        button.hover_style.text.align = alignment;
        button.down_style.text.align = alignment;
        button.show();
    });
}

fn main() {
    bootstrap::start(run as fn());
}

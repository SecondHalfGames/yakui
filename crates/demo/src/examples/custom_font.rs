use yakui::widgets::Text;
use yakui::{column, text, Color};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    column(|| {
        text(32.0, "Default Font");

        let mut text = Text::new(32.0, "Custom Font");
        text.style.font = "monospace".into();
        text.style.color = Color::GREEN;
        text.show();
    });
}

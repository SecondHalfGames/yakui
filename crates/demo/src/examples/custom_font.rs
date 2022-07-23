use yakui::widgets::Text;
use yakui::{column, text, Color3};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    column(|| {
        text(32.0, "Default Font");

        let mut text = Text::new(32.0, "Custom Font");
        text.style.font = "monospace".into();
        text.style.color = Color3::GREEN;
        text.show();
    });
}

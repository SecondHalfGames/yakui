use yakui::widgets::Text;
use yakui::{column, label, row, text, Color3};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    column(|| {
        row(|| {
            label("Hello, world!");

            let mut text = Text::new(48.0, "colored text!");
            text.style.color = Color3::RED;
            text.show();
        });

        text(96.0, "yakui text demo!");
    });
}

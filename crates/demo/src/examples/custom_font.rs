use yakui::widgets::Text;
use yakui::Color3;

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    yakui::column(|| {
        yakui::text(32.0, "Default Font");

        let mut text = Text::new(32.0, "Custom Font");
        text.font = "monospace".into();
        text.color = Color3::GREEN;
        text.show();
    });
}

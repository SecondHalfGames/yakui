use yakui::widgets::Text;
use yakui::Color3;

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    yakui::column(|| {
        yakui::row(|| {
            yakui::label("Hello, world!");

            let mut text = Text::new(48.0, "colored text!".into());
            text.color = Color3::RED;
            text.show();
        });

        yakui::text(96.0, "yakui text demo!");
    });
}

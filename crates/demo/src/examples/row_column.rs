use yakui::Color3;

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    yakui::column(|| {
        yakui::colored_box(Color3::RED, [100.0, 100.0]);
        yakui::row(|| {
            yakui::colored_box(Color3::GREEN, [100.0, 100.0]);
            yakui::colored_box(Color3::REBECCA_PURPLE, [100.0, 100.0]);
            yakui::colored_box(Color3::YELLOW, [100.0, 100.0]);
        });
        yakui::colored_box(Color3::BLUE, [100.0, 100.0]);
    });
}

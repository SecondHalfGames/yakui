use yakui::widgets::Window;

use crate::ExampleState;

pub fn run(_state: &ExampleState) {
    Window::new([300.0, 200.0]).show(|| {
        yakui::center(|| {
            yakui::text(32.0, "Window body!");
        });
    });
}

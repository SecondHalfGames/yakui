use yakui::widgets::Window;
use yakui::{center, text};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    Window::new([300.0, 200.0]).show(|| {
        center(|| {
            text(32.0, "Window body!");
        });
    });
}

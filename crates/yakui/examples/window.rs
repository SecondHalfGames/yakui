use yakui::widgets::Window;
use yakui::{center, text};

pub fn run() {
    Window::new([300.0, 200.0]).show(|| {
        center(|| {
            text(32.0, "Window body!");
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}

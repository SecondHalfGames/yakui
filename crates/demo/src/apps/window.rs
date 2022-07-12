use crate::AppState;

pub fn app(_state: &AppState) {
    yakui::Window::new([300.0, 200.0]).show(|| {
        yakui::center(|| {
            yakui::text(32.0, "Window body!");
        });
    });
}

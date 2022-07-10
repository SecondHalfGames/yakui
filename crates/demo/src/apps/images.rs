use yakui::Vec2;

use crate::AppState;

pub fn app(state: &AppState) {
    yakui::center(|| {
        if state.time.floor() % 2.0 == 0.0 {
            yakui::image(state.monkey, Vec2::new(400.0, 400.0));
        }
    });
}

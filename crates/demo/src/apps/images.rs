use yakui::Vec2;

use crate::AppState;

pub fn app(state: &AppState) {
    yakui::center(|| {
        if f32::sin(state.time * 10.0) > 0.0 {
            yakui::image(state.monkey, Vec2::new(400.0, 400.0));
        }
    });
}

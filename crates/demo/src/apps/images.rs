use yakui::Vec2;

use crate::AppState;

pub fn app(state: &AppState) {
    yakui::center(|| {
        yakui::column(|| {
            if state.time.sin() > 0.0 {
                yakui::text(32.0, format!("{}", state.time.sin()));
                yakui::image(state.monkey, Vec2::new(400.0, 400.0));
            }
        });
    });
}

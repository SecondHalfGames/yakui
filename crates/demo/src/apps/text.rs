use crate::AppState;

pub fn app(_state: &AppState) {
    yakui::column(|| {
        yakui::text(32.0, "Hello, world!");
    });
}

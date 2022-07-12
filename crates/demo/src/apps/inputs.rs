use crate::AppState;

pub fn app(_state: &AppState) {
    yakui::center(|| {
        yakui::button("Button");
    });
}

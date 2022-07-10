use crate::AppState;

pub fn app(_state: &AppState) {
    yakui::column(|| {
        yakui::row(|| {
            yakui::text(32.0, "Hello, world!");
            yakui::text(16.0, "yakui text demo");
        });

        yakui::text(96.0, "yakui text demo!");
    });
}

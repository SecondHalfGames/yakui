use yakui::{Button, Padding};

use crate::AppState;

pub fn app(_state: &AppState) {
    yakui::column(|| {
        let res = yakui::button(Button::styled("First button"));
        if res.clicked {
            println!("Clicked the first button!");
        }

        let padding = Padding::even(8.0);
        yakui::pad(padding, || {
            yakui::row(|| {
                yakui::button(Button::styled("Hello"));
                yakui::button(Button::styled("World"));
                yakui::button(Button::styled("I'm Yakui!"));
            });
        });

        yakui::button(Button::styled("Sincerely, Yakui"));
    });
}

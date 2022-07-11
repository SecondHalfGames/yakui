use yakui::Pad;

use crate::AppState;

pub fn app(_state: &AppState) {
    yakui::column(|| {
        let res = yakui::button("First button");
        if res.clicked {
            println!("Clicked the first button!");
        }

        let padding = Pad::even(8.0);
        yakui::pad(padding, || {
            let mut row = yakui::List::horizontal();
            row.item_spacing = 8.0;
            row.show(|| {
                yakui::button("Hello");
                yakui::button("World");
                yakui::button("I'm Yakui!");
            });
        });

        yakui::button("Sincerely, Yakui");
    });
}

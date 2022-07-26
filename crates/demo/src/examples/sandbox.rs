use yakui::widgets::{Button, Pad};
use yakui::{button, column, pad, row, Color};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    column(|| {
        let res = button("First button");
        if res.clicked {
            println!("Clicked the first button!");
        }

        let padding = Pad::all(8.0);
        pad(padding, || {
            row(|| {
                button("Hello");

                let mut big_button = Button::styled("World".into());
                big_button.text_style.color = Color::RED;
                big_button.text_style.font_size = 30.0;
                big_button.show();

                button("I'm Yakui!");
            });
        });

        button("Sincerely, Yakui");
    });
}

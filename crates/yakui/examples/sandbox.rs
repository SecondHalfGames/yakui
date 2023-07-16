use yakui::widgets::{Button, Pad};
use yakui::{button, column, pad, row, Color};

pub fn run() {
    column(|| {
        let res = button("First button");
        if res.clicked {
            println!("Clicked the first button!");
        }

        let padding = Pad::all(8.0);
        pad(padding, || {
            row(|| {
                button("Hello");

                let mut big_button = Button::styled("World");
                big_button.style.text.color = Color::RED;
                big_button.style.text.font_size = 30.0;
                big_button.show();

                button("I'm Yakui!");
            });
        });

        button("Sincerely, Yakui");
    });
}

fn main() {
    bootstrap::start(run as fn());
}

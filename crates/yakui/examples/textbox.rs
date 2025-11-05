use bootstrap::load_common_fonts;
use yakui::widgets::{Pad, TextBox};
use yakui::{button, center, column, label, row, use_state};

pub fn run() {
    load_common_fonts();

    let text = use_state(|| "".to_owned());
    let align = use_state(|| yakui::style::TextAlignment::Start);

    column(|| {
        label("Align:");

        row(|| {
            if button("Start").clicked {
                align.set(yakui::style::TextAlignment::Start);
            }
            if button("Center").clicked {
                align.set(yakui::style::TextAlignment::Center);
            }
            if button("End").clicked {
                align.set(yakui::style::TextAlignment::End);
            }
        });

        center(|| {
            let mut my_box = TextBox::new();
            my_box.min_width = 400.0;
            my_box.style.font_size = 60.0;
            my_box.style.align = align.get();
            my_box.placeholder_style.font_size = 60.0;
            my_box.placeholder_style.align = align.get();
            my_box.padding = Pad::all(50.0);
            my_box.placeholder = "placeholder".into();
            my_box.multiline = true;

            let response = my_box.show(text.borrow().as_str()).into_inner();
            if let Some(new_text) = response.text {
                text.set(new_text);
            }
            if response.activated {
                println!("{}", text.borrow());
            }
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}

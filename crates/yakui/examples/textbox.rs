use yakui::widgets::{Pad, TextBox};
use yakui::{button, center, expanded, row, use_state};

pub fn run() {
    let text = use_state(|| "Hello!".to_owned());

    // println!("Text: {}", text.borrow());

    center(|| {
        row(|| {
            expanded(|| {
                let mut my_box = TextBox::new(text.borrow().to_owned());
                // my_box.style.font_size = 60.0;
                // my_box.padding = Pad::all(20.0);
                my_box.placeholder = "placeholder".into();

                let response = my_box.show().into_inner();
                if let Some(new_text) = response.text {
                    text.set(new_text);
                }
                if response.activated {
                    println!("{}", text.borrow());
                }
            });

            if button("Reset Text").clicked {
                text.set("Hello!".to_owned());
            }
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}

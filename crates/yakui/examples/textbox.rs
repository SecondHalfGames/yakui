use yakui::style::TextStyle;
use yakui::widgets::{Pad, TextBox};
use yakui::{center, use_state};

pub fn run() {
    let text = use_state(|| "".to_owned());

    center(|| {
        let my_box = TextBox::new(text.borrow().as_str())
            .padding(Pad::all(50.0))
            .style(TextStyle::label().font_size(60.0))
            .placeholder("placeholder");

        let response = my_box.show().into_inner();
        if let Some(new_text) = response.text {
            text.set(new_text);
        }
        if response.activated {
            println!("{}", text.borrow());
        }
    });
}

fn main() {
    bootstrap::start(run as fn());
}

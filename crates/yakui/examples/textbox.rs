use yakui::widgets::{Pad, TextBox};
use yakui::{center, use_state};

pub fn run() {
    let text = use_state(|| "Hello\nWorld\nBar".to_owned());

    center(|| {
        let mut my_box = TextBox::new(text.borrow().as_str());
        my_box.style.font_size = 60.0;
        my_box.padding = Pad::all(50.0);

        if let Some(new_text) = my_box.show().into_inner().text {
            text.set(new_text);
        }
    });
}

fn main() {
    bootstrap::start(run as fn());
}

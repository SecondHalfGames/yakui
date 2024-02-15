use yakui::widgets::{Pad, TextBox};
use yakui::{center, use_state};

pub fn run() {
    let text = use_state(|| "".to_owned());
    let autofocus = use_state(|| false);

    center(|| {
        let mut box1 = TextBox::new(text.borrow().as_str());
        box1.style.font_size = 60.0;
        box1.padding = Pad::all(50.0);
        box1.placeholder = "placeholder".into();

        let response = box1.show();

        if !autofocus.get() {
            autofocus.set(true);
            response.request_focus();
        }

        if let Some(new_text) = response.into_inner().text {
            text.set(new_text);
        }
    });
}

fn main() {
    bootstrap::start(run as fn());
}

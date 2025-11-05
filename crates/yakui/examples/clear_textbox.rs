use yakui::{button, column, textbox, use_state};

fn run() {
    column(|| {
        let text = use_state(|| String::from("Hello"));

        let res = textbox(text.borrow().as_str());
        if let Some(new_text) = res.into_inner().text {
            text.set(new_text);
        }

        if button("Clear").clicked {
            text.borrow_mut().clear();
        }
    });
}

fn main() {
    bootstrap::start(run as fn());
}

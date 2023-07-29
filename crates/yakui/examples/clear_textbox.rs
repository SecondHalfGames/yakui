use yakui::{button, column, textbox, use_state};

fn run() {
    column(|| {
        let text = use_state(|| "Hello".to_string());

        let res = textbox(text.borrow().clone());
        if let Some(new_text) = res.into_inner().text {
            *text.borrow_mut() = new_text;
        }

        if button("Clear").clicked {
            text.borrow_mut().clear();
        }
    });
}

fn main() {
    bootstrap::start(run as fn());
}

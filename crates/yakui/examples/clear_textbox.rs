use yakui::{button, column, textbox, use_state};

fn run() {
    column(|| {
        let text = use_state(String::new);
        let clear = use_state(|| false);

        if clear.get() {
            text.borrow_mut().clear();
            clear.set(false);
        }

        let res = textbox(&text.borrow());
        if let Some(new_text) = res.into_inner().text {
            text.set(new_text);
        }

        if button("Clear").clicked {
            clear.set(true);
        }
    });
}

fn main() {
    bootstrap::start(run as fn());
}

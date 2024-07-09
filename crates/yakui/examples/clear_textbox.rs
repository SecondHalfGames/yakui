use yakui::{button, column, textbox, use_state};

fn run() {
    column(|| {
        let text = use_state(|| String::new());
        let clear = use_state(|| false);

        let res = textbox("Hello", if clear.get() { Some("") } else { None });
        clear.set(false);
        text.set(res.into_inner().text);

        if button("Clear").clicked {
            clear.set(true);
        }
    });
}

fn main() {
    bootstrap::start(run as fn());
}

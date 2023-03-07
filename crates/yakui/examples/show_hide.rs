use yakui::widgets::List;
use yakui::{button, checkbox, use_state};

pub fn run() {
    let shown = use_state(|| false);

    let mut col = List::column();
    col.item_spacing = 8.0;
    col.show(|| {
        shown.set(checkbox(shown.get()).checked);
        if shown.get() {
            button("Hello!");
        }

        button("World!");
    });
}

fn main() {
    bootstrap::start(run as fn());
}

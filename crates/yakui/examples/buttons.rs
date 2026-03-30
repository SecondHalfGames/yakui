use yakui::widgets::List;
use yakui::{button, row, CrossAxisAlignment};

pub fn run() {
    row(|| {
        button("Not stretched");
        List::column()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .show(|| {
                button("Button 1");
                button("Button 2");
                button("Button 3");
            });
    });
}

fn main() {
    bootstrap::start(run as fn());
}

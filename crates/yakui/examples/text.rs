use yakui::style::TextStyle;
use yakui::widgets::Text;
use yakui::{column, label, row, text, Color};

pub fn run() {
    column(|| {
        row(|| {
            label("Hello, world!");

            Text::with_style(
                "colored text!",
                TextStyle::label().font_size(48.0).color(Color::RED),
            )
            .show();
        });

        text(96.0, "yakui text demo!");
    });
}

fn main() {
    bootstrap::start(run as fn());
}

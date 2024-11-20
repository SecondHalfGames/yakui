use yakui::cosmic_text::FamilyOwned;
use yakui::widgets::Text;
use yakui::{column, text, Color};

pub fn run() {
    column(|| {
        // The default font for text is the application-wide "sans-serif" font.
        text(32.0, "Default Font");

        // Fonts can be named by their type, like sans-serif or monospace
        let mut text = Text::new(32.0, "Custom Font");
        text.style.attrs.family_owned = FamilyOwned::Monospace;
        text.style.color = Color::GREEN;
        text.show();

        // ...or you can name the font family directly
        let mut text = Text::new(32.0, "Custom Font (by name)");
        text.style.attrs.family_owned = FamilyOwned::Name("Hack".to_owned());
        text.style.color = Color::GREEN;
        text.show();
    });
}

fn main() {
    bootstrap::start(run as fn());
}

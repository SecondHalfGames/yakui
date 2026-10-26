use std::borrow::Cow;

use yakui::util::read_scope;
use yakui::widgets::{Button, List, Pad, Scope};
use yakui::Color;

pub fn run() {
    List::column().item_spacing(16.0).show(|| {
        styled_button("Default theme");

        with_theme(Theme::light(), || {
            styled_button("Light theme");
        });

        with_theme(Theme::dark(), || {
            styled_button("Dark theme");
        });
    });
}

struct Theme {
    fill_color: Color,
    text_color: Color,
    button_padding: Pad,
}

impl Theme {
    fn light() -> Self {
        Self {
            fill_color: Color::WHITE,
            text_color: Color::BLACK,
            button_padding: Pad::balanced(16.0, 8.0),
        }
    }

    fn dark() -> Self {
        Self {
            fill_color: Color::BLACK,
            text_color: Color::WHITE,
            button_padding: Pad::balanced(16.0, 8.0),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

fn with_theme(theme: Theme, children: impl FnOnce()) {
    Scope::new(theme).show(children);
}

fn styled_button(text: impl Into<Cow<'static, str>>) {
    let theme = read_scope::<Theme>().unwrap_or_default();

    let mut button = Button::styled(text);
    button.padding = theme.button_padding;

    button.style.fill = theme.fill_color;
    button.down_style.fill = theme.fill_color;
    button.hover_style.fill = theme.fill_color;

    button.style.text.color = theme.text_color;
    button.down_style.text.color = theme.text_color;
    button.hover_style.text.color = theme.text_color;

    button.show();
}

fn main() {
    bootstrap::start(run as fn());
}

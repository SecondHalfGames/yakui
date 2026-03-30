use bootstrap::ExampleState;
use yakui::style::TextStyle;
use yakui::widgets::{Pad, TextBox};
use yakui::{center, use_state};

pub fn run(state: &mut ExampleState) {
    let text = use_state(String::new);
    let autofocus = use_state(|| false);

    center(|| {
        let box1 = TextBox::new(text.borrow().clone())
            .style(TextStyle::label().font_size(60.0))
            .padding(Pad::all(50.0))
            .placeholder("placeholder");

        let response = box1.show();

        if !autofocus.get() {
            autofocus.set(true);

            let focus_id = response.id;
            state.commands.push(Box::new(move |yakui| {
                yakui.request_focus(Some(focus_id));
            }));
        }

        if let Some(new_text) = response.into_inner().text {
            text.set(new_text);
        }
    });
}

fn main() {
    bootstrap::start(run as fn(&mut ExampleState));
}

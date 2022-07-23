use yakui::{button, center, checkbox, column, textbox};

use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    center(|| {
        column(|| {
            if button("Button").clicked {
                println!("Button clicked");
            }

            state.checked = checkbox(state.checked).checked;

            if let Some(new_name) = textbox(18.0, &state.name).text.as_ref() {
                state.name = new_name.clone();
            }
        });
    });
}

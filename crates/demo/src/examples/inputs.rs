use yakui::{button, checkbox, column, textbox};

use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    column(|| {
        if button("Button").clicked {
            println!("Button clicked");
        }

        state.checked = checkbox(state.checked).checked;

        if let Some(new_name) = textbox(&state.name).text.as_ref() {
            state.name = new_name.clone();
        }
    });
}

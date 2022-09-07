use yakui::{button, checkbox, column, textbox, use_state};

use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    let checked = use_state(|| false);

    column(|| {
        if button("Button").clicked {
            println!("Button clicked");
        }

        let res = checkbox(checked.get());
        checked.set(res.checked);

        if let Some(new_name) = textbox(&state.name).text.as_ref() {
            state.name = new_name.clone();
        }
    });
}

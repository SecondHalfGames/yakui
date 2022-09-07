use yakui::{button, checkbox, column, textbox, use_state};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    let checked = use_state(|| false);
    let name = use_state(|| String::from("Hello"));

    column(|| {
        if button("Button").clicked {
            println!("Button clicked");
        }

        let res = checkbox(checked.get());
        checked.set(res.checked);

        let res = textbox(name.borrow().clone());
        if let Some(new_name) = res.text.as_ref() {
            name.set(new_name.clone());
        }
    });
}

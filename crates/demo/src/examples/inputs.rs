use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    yakui::center(|| {
        yakui::column(|| {
            if yakui::button("Button").clicked {
                println!("Button clicked");
            }

            state.checked = yakui::checkbox(state.checked).checked;

            if let Some(new_name) = yakui::textbox(18.0, &state.name).text.as_ref() {
                state.name = new_name.clone();
            }
        });
    });
}

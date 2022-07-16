use crate::ExampleState;

pub fn run(_state: &ExampleState) {
    yakui::center(|| {
        yakui::button("Button");
    });
}

use yakui::{button, center, column, label, use_state};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    let value = use_state(|| 0);

    center(|| {
        column(|| {
            label(format!("Value: {}", value.get()));

            if button("Increment").clicked {
                *value.borrow_mut() += 1;
            }
        });
    });
}

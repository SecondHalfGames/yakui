use yakui::{button, center};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    let count = use_state(|| 0);

    center(|| if button(format!("Count: {count}")).clicked {});
}

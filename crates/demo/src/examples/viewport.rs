use yakui::{colored_box, Color};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    colored_box(Color::hex(0x888888), [10.0, 10.0]);
}

use yakui::{colored_box, draggable, offset, Color, Vec2};

use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    offset(state.pos + state.offset, || {
        let res = draggable(|| {
            colored_box(Color::RED, Vec2::splat(100.0));
        });

        if let Some(new) = res.into_inner().dragging {
            state.offset = new.current - new.start;
        } else if state.offset != Vec2::ZERO {
            state.pos += state.offset;
            state.offset = Vec2::ZERO;
        }
    });
}

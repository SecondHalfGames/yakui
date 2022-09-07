use yakui::{colored_box, draggable, offset, use_state, Color, Vec2};

use crate::ExampleState;

pub fn run(_state: &mut ExampleState) {
    let pos = use_state(|| Vec2::ZERO);
    let extra = use_state(|| Vec2::ZERO);

    offset(pos.get() + extra.get(), || {
        let res = draggable(|| {
            colored_box(Color::RED, Vec2::splat(100.0));
        });

        if let Some(new) = res.into_inner().dragging {
            extra.set(new.current - new.start);
        } else if extra.get() != Vec2::ZERO {
            pos.modify(|pos| pos + extra.get());
            extra.set(Vec2::ZERO);
        }
    });
}

use yakui::{Color, Vec2, colored_box, draggable, offset, use_state};

pub fn run() {
    let pos = use_state(|| Vec2::ZERO);

    offset(pos.get(), || {
        let res = draggable(|| {
            colored_box(Color::RED, Vec2::splat(100.0));
        });

        if let Some(new) = res.dragging {
            pos.set(new.current);
        }
    });
}

fn main() {
    bootstrap::start(run as fn());
}

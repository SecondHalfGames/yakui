use glam::Vec2;

use crate::{context::Context, snapshot::Element};

pub(crate) struct Layout {
    pub direction: Direction,
}

impl Layout {
    pub fn vertical() -> Self {
        Self {
            direction: Direction::Down,
        }
    }
}

pub(crate) struct FixedSizeBox {
    pub size: Vec2,
}

pub(crate) enum Direction {
    Down,
    Right,
}

pub fn vertical<F: FnOnce()>(contents: F) {
    let context = Context::active();

    let id = context
        .borrow_mut()
        .snapshot_mut()
        .push(Element::new::<Layout, Layout>(Layout::vertical()));

    contents();

    context.borrow_mut().snapshot_mut().pop(id);
}

pub fn fsbox<S: Into<Vec2>>(size: S) {
    let context = Context::active();

    let size = size.into();
    context
        .borrow_mut()
        .snapshot_mut()
        .insert(Element::new::<FixedSizeBox, FixedSizeBox>(FixedSizeBox {
            size,
        }));
}

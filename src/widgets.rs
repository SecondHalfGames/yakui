use glam::Vec2;

use crate::component::Component;
use crate::context::Context;
use crate::snapshot::Element;

#[derive(Debug, Clone)]
pub struct Layout {
    pub direction: Direction,
}

impl Layout {
    pub fn vertical() -> Self {
        Self {
            direction: Direction::Down,
        }
    }
}

impl Component<Layout> for Layout {
    fn new(props: &Layout) -> Self {
        props.clone()
    }

    fn update(&mut self, props: &Layout) {
        *self = props.clone();
    }
}

#[derive(Debug, Clone)]
pub struct FixedSizeBox {
    pub size: Vec2,
}

impl Component<FixedSizeBox> for FixedSizeBox {
    fn new(props: &FixedSizeBox) -> Self {
        props.clone()
    }

    fn update(&mut self, props: &FixedSizeBox) {
        *self = props.clone();
    }
}

#[derive(Debug, Clone)]
pub enum Direction {
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

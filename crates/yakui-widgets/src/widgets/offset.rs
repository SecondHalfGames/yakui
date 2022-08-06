use yakui_core::dom::Dom;
use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::util::widget_children;

/**
Offsets its child by the given number of logical pixels.
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Offset {
    pub offset: Vec2,
}

impl Offset {
    pub fn new(offset: Vec2) -> Self {
        Self { offset }
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<OffsetWidget> {
        widget_children::<OffsetWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct OffsetWidget {
    props: Offset,
}

pub type OffsetResponse = ();

impl Widget for OffsetWidget {
    type Props = Offset;
    type Response = OffsetResponse;

    fn new() -> Self {
        Self {
            props: Offset::new(Vec2::ZERO),
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get_current();

        // Offset allows its children to be smaller than the minimum size
        // enforced by the incoming constraints.
        let constraints = Constraints::loose(input.max);

        let mut self_size = if input.max.is_finite() {
            input.max
        } else if input.min.is_finite() {
            input.min
        } else {
            Vec2::ZERO
        };

        for &child in &node.children {
            let child_size = layout.calculate(dom, child, constraints);
            self_size = self_size.max(child_size);
            layout.set_pos(child, self.props.offset);
        }

        self_size
    }
}

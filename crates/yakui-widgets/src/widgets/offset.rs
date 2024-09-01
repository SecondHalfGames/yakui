use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::Response;

use crate::util::widget_children;

/**
Offsets its child by the given number of logical pixels.
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Offset {
    pub offset: Vec2,
}

impl Offset {
    pub fn new(offset: Vec2) -> Self {
        Self { offset }
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<OffsetResponse> {
        widget_children::<OffsetWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct OffsetWidget {
    props: Offset,
}

pub type OffsetResponse = ();

impl Widget for OffsetWidget {
    type Props<'a> = Offset;
    type Response = OffsetResponse;

    fn new() -> Self {
        Self {
            props: Offset::new(Vec2::ZERO),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();

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
            let child_size = ctx.calculate_layout(child, constraints);
            self_size = self_size.max(child_size);
            ctx.layout.set_pos(child, self.props.offset);
        }

        self_size
    }
}

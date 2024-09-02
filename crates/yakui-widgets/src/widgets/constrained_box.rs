use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::Response;

use crate::util::widget_children;

/**
A box that forces specific constraints onto its child.

Responds with [ConstrainedBoxResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct ConstrainedBox {
    pub constraints: Constraints,
}

impl ConstrainedBox {
    pub fn new(constraints: Constraints) -> Self {
        Self { constraints }
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<ConstrainedBoxResponse> {
        widget_children::<ConstrainedBoxWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct ConstrainedBoxWidget {
    props: ConstrainedBox,
}

pub type ConstrainedBoxResponse = ();

impl Widget for ConstrainedBoxWidget {
    type Props<'a> = ConstrainedBox;
    type Response = ConstrainedBoxResponse;

    fn new() -> Self {
        Self {
            props: ConstrainedBox::new(Constraints {
                min: Vec2::ZERO,
                max: Vec2::ZERO,
            }),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = Vec2::ZERO;
        let constraints = Constraints {
            min: Vec2::max(input.min, self.props.constraints.min),
            max: Vec2::min(input.max, self.props.constraints.max),
        };

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, constraints);
            size = size.max(child_size);
        }

        input.constrain(constraints.constrain(size))
    }
}

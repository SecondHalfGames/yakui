use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::Response;

use crate::auto_builders;
use crate::util::widget_children;

/**
A box that renders its child with one or both of its constraint axes ignored.

Responds with [UnconstrainedBoxResponse].
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct UnconstrainedBox {
    pub constrain_x: bool,
    pub constrain_y: bool,
}

auto_builders!(UnconstrainedBox {
    constrain_x: bool,
    constrain_y: bool,
});

impl UnconstrainedBox {
    pub fn new() -> Self {
        Self {
            constrain_x: false,
            constrain_y: false,
        }
    }

    #[track_caller]
    pub fn show<F: FnOnce()>(self, children: F) -> Response<UnconstrainedBoxResponse> {
        widget_children::<UnconstrainedBoxWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct UnconstrainedBoxWidget {
    props: UnconstrainedBox,
}

pub type UnconstrainedBoxResponse = ();

impl Widget for UnconstrainedBoxWidget {
    type Props<'a> = UnconstrainedBox;
    type Response = UnconstrainedBoxResponse;

    fn new() -> Self {
        Self {
            props: UnconstrainedBox::new(),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();

        let (min_x, max_x) = if self.props.constrain_x {
            (0.0, input.max.x)
        } else {
            (0.0, f32::INFINITY)
        };

        let (min_y, max_y) = if self.props.constrain_y {
            (0.0, input.max.y)
        } else {
            (0.0, f32::INFINITY)
        };

        let constraints = Constraints {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        };

        let mut size = Vec2::ZERO;
        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, constraints);
            size = size.max(child_size);
        }

        input.constrain_min(size)
    }
}

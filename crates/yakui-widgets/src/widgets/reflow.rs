use yakui_core::geometry::{Constraints, Dim2, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::{Alignment, Flow, Pivot, Response};

use crate::util::widget_children;

/**
Changes the flow behavior a widget tree, allowing it to break out of list, grid,
or table layouts.
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Reflow {
    pub anchor: Alignment,
    pub pivot: Pivot,
    pub offset: Dim2,
}

impl Reflow {
    pub fn new(anchor: Alignment, pivot: Pivot, offset: Dim2) -> Self {
        Self {
            anchor,
            pivot,
            offset,
        }
    }

    #[track_caller]
    pub fn show<F: FnOnce()>(self, children: F) -> Response<ReflowResponse> {
        widget_children::<ReflowWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct ReflowWidget {
    props: Reflow,
}

pub type ReflowResponse = ();

impl Widget for ReflowWidget {
    type Props<'a> = Reflow;
    type Response = ReflowResponse;

    fn new() -> Self {
        Self {
            props: Reflow {
                anchor: Alignment::TOP_LEFT,
                pivot: Pivot::TOP_LEFT,
                offset: Dim2::ZERO,
            },
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn flow(&self) -> Flow {
        Flow::Relative {
            anchor: self.props.anchor,
            offset: self.props.offset,
        }
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, _constraints: Constraints) -> Vec2 {
        ctx.layout.new_layer(ctx.dom);
        ctx.layout.escape_clipping(ctx.dom);

        let node = ctx.dom.get_current();
        let mut size = Vec2::ZERO;
        for &child in &node.children {
            size = size.max(ctx.calculate_layout(child, Constraints::none()));
        }

        let pivot_offset = -size * self.props.pivot.as_vec2();
        for &child in &node.children {
            ctx.layout.set_pos(child, pivot_offset);
        }

        Vec2::ZERO
    }
}

use yakui_core::geometry::{Constraints, Dim2, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::{Alignment, Flow, Response};

use crate::util::widget_children;

/**
Changes the flow behavior a widget tree, allowing it to break out of list, grid,
or table layouts.
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Reflow {
    pub anchor: Alignment,
    pub offset: Dim2,
}

impl Reflow {
    pub fn new(anchor: Alignment, offset: Dim2) -> Self {
        Self { anchor, offset }
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<ReflowWidget> {
        widget_children::<ReflowWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct ReflowWidget {
    props: Reflow,
}

pub type ReflowResponse = ();

impl Widget for ReflowWidget {
    type Props = Reflow;
    type Response = ReflowResponse;

    fn new() -> Self {
        Self {
            props: Reflow {
                anchor: Alignment::TOP_LEFT,
                offset: Dim2::ZERO,
            },
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn flow(&self) -> Flow {
        Flow::Relative {
            anchor: self.props.anchor,
            offset: self.props.offset,
        }
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, _constraints: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        for &child in &node.children {
            ctx.calculate_layout(child, Constraints::none());
        }

        Vec2::ZERO
    }
}

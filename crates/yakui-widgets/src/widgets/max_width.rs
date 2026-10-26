use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::Response;

use crate::util::widget_children;

/**
A box that enforces a maximum width upon its children.

Responds with [MaxWidthResponse].
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct MaxWidth {
    pub max_width: f32,
}

impl MaxWidth {
    pub fn new(max_width: f32) -> Self {
        Self { max_width }
    }

    #[track_caller]
    pub fn show<F: FnOnce()>(self, children: F) -> Response<MaxWidthResponse> {
        widget_children::<MaxWidthWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct MaxWidthWidget;

pub type MaxWidthResponse = ();

impl Widget for MaxWidthWidget {
    type Props = MaxWidth;
    type Response = MaxWidthResponse;

    fn new(_props: &Self::Props) -> Self {
        Self
    }

    fn update(&mut self, _props: &Self::Props) -> Self::Response {}

    fn layout(
        &self,
        props: &Self::Props,
        mut ctx: LayoutContext<'_>,
        mut constraints: Constraints,
    ) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = Vec2::ZERO;
        constraints.max.x = constraints.max.x.min(props.max_width);

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, constraints);
            size = size.max(child_size);
        }

        size
    }
}

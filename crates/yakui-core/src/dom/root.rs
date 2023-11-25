use crate::geometry::Constraints;
use crate::widget::{LayoutContext, Widget};

#[derive(Debug)]
pub struct RootWidget;

impl Widget for RootWidget {
    type Props<'a> = ();
    type Response = ();

    fn new() -> Self {
        Self
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {}

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> glam::Vec2 {
        ctx.layout.new_layer(ctx.dom);

        let node = ctx.dom.get_current();

        for &child in &node.children {
            ctx.calculate_layout(child, constraints);
        }

        constraints.max
    }
}

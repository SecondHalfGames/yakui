use crate::geometry::Constraints;
use crate::widget::{LayoutContext, Widget};

#[derive(Debug)]
pub struct RootWidget;

impl Widget for RootWidget {
    type Props = ();
    type Response = ();

    fn new() -> Self {
        Self
    }

    fn update(&mut self, _props: Self::Props) -> Self::Response {}

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> glam::Vec2 {
        let node = ctx.dom.get_current();

        for &child in &node.children {
            ctx.calculate_layout(child, constraints);
        }

        constraints.max
    }
}

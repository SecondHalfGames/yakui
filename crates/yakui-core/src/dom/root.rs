use crate::geometry::Constraints;
use crate::layout::LayoutDom;
use crate::widget::Widget;

use super::Dom;

#[derive(Debug)]
pub struct RootWidget;

impl Widget for RootWidget {
    type Props = ();
    type Response = ();

    fn new() -> Self {
        Self
    }

    fn update(&mut self, _props: Self::Props) -> Self::Response {}

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> glam::Vec2 {
        let node = dom.get_current();

        for &child in &node.children {
            layout.calculate(dom, child, constraints);
        }

        constraints.max
    }
}

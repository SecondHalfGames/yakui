use yakui_core::{
    dom::{Dom, LayoutDom},
    draw, Component, Constraints, Index, Vec2,
};

use crate::util::component_children;

#[derive(Debug)]
pub struct Pad {
    index: Index,
    props: PadProps,
}

#[derive(Debug, Clone)]
pub struct PadProps {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

pub type PadResponse = ();

impl Component for Pad {
    type Props = PadProps;
    type Response = PadResponse;

    fn new(index: Index, props: Self::Props) -> Self {
        Self { index, props }
    }

    fn update(&mut self, props: &Self::Props) {
        self.props = props.clone();
    }

    fn size(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get(self.index).unwrap();

        let mut self_size = Vec2::ZERO;

        let total_padding = Vec2::new(
            self.props.left + self.props.right,
            self.props.top + self.props.bottom,
        );
        let offset = Vec2::new(self.props.left, self.props.top);

        let child_constraints = Constraints {
            min: input.min - total_padding,
            max: input.max - total_padding,
        };

        for &child in &node.children {
            self_size = layout.calculate(dom, child, child_constraints) + total_padding;
            layout.set_pos(child, offset);
        }

        input.constrain(self_size)
    }

    fn draw(&self, dom: &Dom, layout: &LayoutDom, output: &mut draw::Output) {
        let node = dom.get(self.index).unwrap();

        for &index in &node.children {
            let child = dom.get(index).unwrap();
            child.component.draw(dom, layout, output);
        }
    }

    fn respond(&mut self) -> Self::Response {}
}

pub fn pad<F: FnOnce()>(props: PadProps, children: F) -> PadResponse {
    component_children::<Pad, _>(children, props)
}

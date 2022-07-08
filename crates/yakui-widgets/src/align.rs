use yakui_core::{
    dom::{Dom, LayoutDom},
    draw, Component, Constraints, Index, Vec2,
};

use crate::{util::component_children, Alignment};

#[derive(Debug)]
pub struct Align {
    index: Index,
    props: AlignProps,
}

#[derive(Debug, Clone)]
pub struct AlignProps {
    pub alignment: Alignment,
}

pub type AlignResponse = ();

impl Component for Align {
    type Props = AlignProps;
    type Response = AlignResponse;

    fn new(index: Index, props: Self::Props) -> Self {
        Self { index, props }
    }

    fn update(&mut self, props: &Self::Props) {
        self.props = props.clone();
    }

    fn size(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get(self.index).unwrap();

        let self_size = input.max;
        let align = self.props.alignment.as_vec2();

        for &child in &node.children {
            let child_size = layout.calculate(dom, child, input);
            layout.set_pos(child, align * self_size - align * child_size);
        }

        self_size
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

pub fn center<F: FnOnce()>(children: F) -> AlignResponse {
    component_children::<Align, _>(
        children,
        AlignProps {
            alignment: Alignment::CENTER,
        },
    )
}

pub fn align<F: FnOnce()>(alignment: Alignment, children: F) -> AlignResponse {
    component_children::<Align, _>(children, AlignProps { alignment })
}

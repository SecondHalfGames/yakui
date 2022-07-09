use yakui_core::{dom::Dom, layout::LayoutDom, paint, Color3, Component, Constraints, Index, Vec2};

use crate::util::component_children;

#[derive(Debug, Clone)]
pub struct ColoredBox {
    pub color: Color3,
}

#[derive(Debug)]
pub struct ColoredBoxComponent {
    index: Index,
    props: ColoredBox,
}

pub type ColoredBoxResponse = ();

impl Component for ColoredBoxComponent {
    type Props = ColoredBox;
    type Response = ColoredBoxResponse;

    fn new(index: Index, props: Self::Props) -> Self {
        Self { index, props }
    }

    fn update(&mut self, props: &Self::Props) {
        self.props = props.clone();
    }

    fn size(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get(self.index).unwrap();

        let mut self_size = Vec2::ZERO;

        for &child in &node.children {
            self_size = layout.calculate(dom, child, input);
        }

        input.constrain(self_size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, output: &mut paint::Output) {
        let node = dom.get(self.index).unwrap();
        let layout_node = layout.get(self.index).unwrap();
        let viewport = layout.viewport;
        let size = layout_node.rect.size() / viewport.size();
        let pos = (layout_node.rect.pos() + viewport.pos()) / viewport.size();

        output.rect(pos, size, self.props.color);

        for &index in &node.children {
            let child = dom.get(index).unwrap();
            child.component.paint(dom, layout, output);
        }
    }

    fn respond(&mut self) -> Self::Response {}
}

pub fn colored_box<F: FnOnce()>(color: Color3, children: F) -> ColoredBoxResponse {
    component_children::<ColoredBoxComponent, _>(children, ColoredBox { color })
}

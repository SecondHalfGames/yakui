use yakui_core::paint::PaintDom;
use yakui_core::{dom::Dom, layout::LayoutDom, Constraints, Vec2, Widget};

use crate::{util::widget_children, Alignment};

#[derive(Debug, Clone)]
pub struct Align {
    pub alignment: Alignment,
}

impl Align {
    pub fn new(alignment: Alignment) -> Self {
        Self { alignment }
    }

    pub fn center() -> Self {
        Self {
            alignment: Alignment::CENTER,
        }
    }

    pub fn show<F: FnOnce()>(self, children: F) -> AlignResponse {
        widget_children::<AlignWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct AlignWidget {
    props: Align,
}

pub type AlignResponse = ();

impl Widget for AlignWidget {
    type Props = Align;
    type Response = AlignResponse;

    fn new(props: Self::Props) -> Self {
        Self { props }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get_current();

        let self_size = input.max;
        let align = self.props.alignment.as_vec2();

        for &child in &node.children {
            let child_size = layout.calculate(dom, child, input);
            layout.set_pos(child, align * self_size - align * child_size);
        }

        self_size
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let node = dom.get_current();
        for &child in &node.children {
            paint.paint(dom, layout, child);
        }
    }

    fn respond(&mut self) -> Self::Response {}
}

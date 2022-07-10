use yakui_core::paint::PaintDom;
use yakui_core::{dom::Dom, layout::LayoutDom, Constraints, Index, Vec2, Widget};

use crate::{util::widget_children, Alignment};

#[derive(Debug, Clone)]
pub struct Align {
    pub alignment: Alignment,
}

#[derive(Debug)]
pub struct AlignComponent {
    props: Align,
}

pub type AlignResponse = ();

impl Widget for AlignComponent {
    type Props = Align;
    type Response = AlignResponse;

    fn new(_index: Index, props: Self::Props) -> Self {
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

pub fn center<F: FnOnce()>(children: F) -> AlignResponse {
    widget_children::<AlignComponent, _>(
        children,
        Align {
            alignment: Alignment::CENTER,
        },
    )
}

pub fn align<F: FnOnce()>(alignment: Alignment, children: F) -> AlignResponse {
    widget_children::<AlignComponent, _>(children, Align { alignment })
}

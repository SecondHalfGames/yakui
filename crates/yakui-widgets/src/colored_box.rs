use yakui_core::paint::{PaintDom, PaintRect};
use yakui_core::Rect;
use yakui_core::{dom::Dom, layout::LayoutDom, Color3, Constraints, Index, Vec2, Widget};

use crate::util::widget_children;

#[derive(Debug, Clone)]
pub struct ColoredBox {
    pub color: Color3,
}

#[derive(Debug)]
pub struct ColoredBoxWidget {
    index: Index,
    props: ColoredBox,
}

pub type ColoredBoxResponse = ();

impl Widget for ColoredBoxWidget {
    type Props = ColoredBox;
    type Response = ColoredBoxResponse;

    fn new(index: Index, props: Self::Props) -> Self {
        Self { index, props }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get(self.index).unwrap();

        let mut self_size = Vec2::ZERO;

        for &child in &node.children {
            self_size = layout.calculate(dom, child, input);
        }

        input.constrain(self_size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let node = dom.get(self.index).unwrap();
        let layout_node = layout.get(self.index).unwrap();
        let viewport = layout.viewport;
        let size = layout_node.rect.size() / viewport.size();
        let pos = (layout_node.rect.pos() + viewport.pos()) / viewport.size();

        let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
        rect.color = self.props.color;
        paint.add_rect(rect);

        for &index in &node.children {
            let child = dom.get(index).unwrap();
            child.widget.paint(dom, layout, paint);
        }
    }

    fn respond(&mut self) -> Self::Response {}
}

pub fn colored_box<F: FnOnce()>(color: Color3, children: F) -> ColoredBoxResponse {
    widget_children::<ColoredBoxWidget, _>(children, ColoredBox { color })
}

use yakui_core::dom::Dom;
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect};
use yakui_core::widget::Widget;
use yakui_core::{Color3, Constraints, Rect, Response, Vec2};

use crate::util::{widget, widget_children};

#[derive(Debug, Clone)]
pub struct ColoredBox {
    pub color: Color3,
    pub min_size: Vec2,
}

impl ColoredBox {
    pub fn sized(color: Color3, size: Vec2) -> Self {
        Self {
            color,
            min_size: size,
        }
    }

    pub fn container(color: Color3) -> Self {
        Self {
            color,
            min_size: Vec2::ZERO,
        }
    }

    pub fn show(self) -> Response<ColoredBoxWidget> {
        widget::<ColoredBoxWidget>(self)
    }

    pub fn show_children<F: FnOnce()>(self, children: F) -> Response<ColoredBoxWidget> {
        widget_children::<ColoredBoxWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct ColoredBoxWidget {
    props: ColoredBox,
}

pub type ColoredBoxResponse = ();

impl Widget for ColoredBoxWidget {
    type Props = ColoredBox;
    type Response = ColoredBoxResponse;

    fn new(props: Self::Props) -> Self {
        Self { props }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get_current();
        let mut size = self.props.min_size;

        for &child in &node.children {
            let child_size = layout.calculate(dom, child, input);
            size = size.max(child_size);
        }

        input.constrain(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let node = dom.get_current();
        let layout_node = layout.get(dom.current()).unwrap();
        let viewport = layout.viewport();
        let size = layout_node.rect.size() / viewport.size();
        let pos = (layout_node.rect.pos() + viewport.pos()) / viewport.size();

        let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
        rect.color = self.props.color;
        paint.add_rect(rect);

        for &child in &node.children {
            paint.paint(dom, layout, child);
        }
    }

    fn respond(&mut self) -> Self::Response {}
}

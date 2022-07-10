use yakui_core::paint::{PaintDom, PaintRect};
use yakui_core::Rect;
use yakui_core::{dom::Dom, layout::LayoutDom, Color3, Constraints, Vec2, Widget};

use crate::util::{widget, widget_children};

#[derive(Debug, Clone)]
pub struct ColoredBox {
    pub color: Color3,
    pub size: Vec2,
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
        let mut size = self.props.size;

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

pub fn colored_box<S: Into<Vec2>>(color: Color3, size: S) -> ColoredBoxResponse {
    widget::<ColoredBoxWidget>(ColoredBox {
        color,
        size: size.into(),
    })
}

pub fn colored_box_container<F: FnOnce()>(color: Color3, children: F) -> ColoredBoxResponse {
    widget_children::<ColoredBoxWidget, F>(
        children,
        ColoredBox {
            color,
            size: Vec2::ZERO,
        },
    )
}

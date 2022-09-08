use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::util::{widget, widget_children};

/**
A colored box that can contain children.

Responds with [ColoredBoxResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ColoredBox {
    pub color: Color,
    pub min_size: Vec2,
}

impl ColoredBox {
    pub fn empty() -> Self {
        Self {
            color: Color::WHITE,
            min_size: Vec2::ZERO,
        }
    }

    pub fn sized(color: Color, size: Vec2) -> Self {
        Self {
            color,
            min_size: size,
        }
    }

    pub fn container(color: Color) -> Self {
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

    fn new() -> Self {
        Self {
            props: ColoredBox::empty(),
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = self.props.min_size;

        for &child in &node.children {
            let child_size = ctx.layout.calculate(ctx.dom, child, input);
            size = size.max(child_size);
        }

        input.constrain_min(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let mut rect = PaintRect::new(layout_node.rect);
        rect.color = self.props.color;
        ctx.paint.add_rect(rect);

        for &child in &node.children {
            ctx.paint(child);
        }
    }
}

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
#[must_use = "yakui widgets do nothing if you don't `show` them"]
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

    pub fn show(self) -> Response<ColoredBoxResponse> {
        widget::<ColoredBoxWidget>(self)
    }

    pub fn show_children<F: FnOnce()>(self, children: F) -> Response<ColoredBoxResponse> {
        widget_children::<ColoredBoxWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct ColoredBoxWidget {
    props: ColoredBox,
}

pub type ColoredBoxResponse = ();

impl Widget for ColoredBoxWidget {
    type Props<'a> = ColoredBox;
    type Response = ColoredBoxResponse;

    fn new() -> Self {
        Self {
            props: ColoredBox::empty(),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = self.props.min_size;

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, input);
            size = size.max(child_size);
        }

        input.constrain_min(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let mut rect = PaintRect::new(layout_node.rect);
        rect.color = self.props.color;
        rect.add(ctx.paint);

        for &child in &node.children {
            ctx.paint(child);
        }
    }
}

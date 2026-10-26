use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::auto_builders;
use crate::util::{widget, widget_children};

/**
A colored box that can contain children.

Responds with [ColoredBoxResponse].
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct ColoredBox {
    pub color: Color,
    pub min_size: Vec2,
}

auto_builders!(ColoredBox {
    color: Color,
    min_size: Vec2,
});

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

    #[track_caller]
    pub fn show(self) -> Response<ColoredBoxResponse> {
        widget::<ColoredBoxWidget>(self)
    }

    #[track_caller]
    pub fn show_children<F: FnOnce()>(self, children: F) -> Response<ColoredBoxResponse> {
        widget_children::<ColoredBoxWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct ColoredBoxWidget;

pub type ColoredBoxResponse = ();

impl Widget for ColoredBoxWidget {
    type Props = ColoredBox;
    type Response = ColoredBoxResponse;

    fn new(_props: &Self::Props) -> Self {
        Self
    }

    fn update(&mut self, _props: &Self::Props) -> Self::Response {}

    fn layout(&self, props: &Self::Props, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = props.min_size;

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, input);
            size = size.max(child_size);
        }

        input.constrain_min(size)
    }

    fn paint(&self, props: &Self::Props, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let mut rect = PaintRect::new(layout_node.rect);
        rect.color = props.color;
        rect.add(ctx.paint);

        for &child in &node.children {
            ctx.paint(child);
        }
    }
}
